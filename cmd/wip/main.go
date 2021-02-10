package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

func main() {
	start := time.Now()
	args := os.Args[1:]
	filename := os.Getenv("WIP_FILENAME")
	if filename == "" {
		filename = filepath.Join(os.Getenv("HOME"), ".wip")
	}
	wip, err := NewWIP(filename)
	if err != nil {
		fmt.Printf("error: %s\n", err.Error())
		os.Exit(-1)
	}
	if len(args) > 0 {
		var err error
		switch args[0] {
		case "push":
			err = wip.Push(Item(strings.Join(args[1:], " ")))
		case "pop":
			err = wip.Pop()
		case "focus":
			index, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				break
			}
			err = wip.Focus(uint(index))
		}
		if err != nil {
			fmt.Printf("error: %s\n", err.Error())
			os.Exit(-1)
		}
	}
	fmt.Print(wip.Show())
	if os.Getenv("WIP_TIMING") == "true" {
		diff := time.Now().Sub(start)
		fmt.Println("done in", diff)
	}
}

type Item string

type WIP struct {
	filename string
	ops      []*Op
}

func NewWIP(filename string) (*WIP, error) {
	file, err := os.Open(filename)
	if err != nil && os.IsNotExist(err) {
		return &WIP{filename: filename, ops: nil}, nil
	} else if err != nil {
		return nil, err
	}
	defer file.Close()
	ops, err := readOps(file)
	if err != nil {
		return nil, err
	}
	return &WIP{
		filename: filename,
		ops:      ops,
	}, nil
}

func (wip *WIP) Push(item Item) error {
	applier := PushApplier(item)
	var op = NewOp(&applier, time.Now())
	return wip.writeOp(op)
}

func (wip *WIP) Pop() error {
	var op = NewOp(&PopApplier{}, time.Now())
	return wip.writeOp(op)
}

func (wip *WIP) Focus(index uint) error {
	applier := FocusApplier(index)
	op := NewOp(&applier, time.Now())
	return wip.writeOp(op)
}

func (wip *WIP) items() []Item {
	var items []Item
	for _, op := range wip.ops {
		items = op.Payload.Apply(items)
	}
	return items
}

func (wip *WIP) Show() string {
	var builder strings.Builder
	items := wip.items()
	if len(items) == 0 {
		builder.WriteString("no WIP\n")
	}
	for len(items) > 0 {
		item := items[len(items) - 1]
		builder.WriteString(fmt.Sprintf("%d: %s\n", len(items) - 1, item))
		items = items[:len(items) - 1]
	}
	return builder.String()
}

func (wip *WIP) writeOp(op *Op) error {
	file, err := os.OpenFile(wip.filename, os.O_WRONLY|os.O_APPEND|os.O_CREATE, 0666)
	if err != nil {
		return err
	}
	defer file.Close()
	jsonLine, err := json.Marshal(op)
	if err != nil {
		return err
	}
	jsonLine = append(jsonLine, '\n')
	_, err = file.Write(jsonLine)
	if err != nil {
		return err
	}
	wip.ops = append(wip.ops, op)
	return nil
}

func readOps(file *os.File) ([]*Op, error) {
	var ops []*Op
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		var op Op
		err := json.Unmarshal(scanner.Bytes(), &op)
		if err != nil {
			return nil, err
		}
		ops = append(ops, &op)
	}
	if err := scanner.Err(); err != nil {
		return nil, err
	}
	return ops, nil
}

type ISO8601 time.Time

const ISO8601Format = "2006-01-02T15:04:05-0700"

func (t *ISO8601) UnmarshalJSON(bytes []byte) error {
	var rawString string
	err := json.Unmarshal(bytes, &rawString)
	if err != nil {
		return err
	}
	raw, err := time.Parse(ISO8601Format, rawString)
	if err != nil {
		return err
	}
	*t = ISO8601(raw)
	return nil
}

func (t *ISO8601) MarshalJSON() ([]byte, error) {
	return json.Marshal(time.Time(*t).Format(ISO8601Format))
}

type Op struct {
	Type       string  `json:"type"`
	OccurredAt ISO8601 `json:"occurred_at"`
	Payload    Applier `json:"payload,omitempty"`
}

func (o *Op) MarshalJSON() ([]byte, error) {
	jOp := &jsonOp{
		Type:       o.Type,
		OccurredAt: o.OccurredAt,
	}
	var err error
	jOp.Payload, err = json.Marshal(o.Payload)
	if err != nil {
		return nil, err
	}
	if bytes.Compare(jOp.Payload, []byte("null")) == 0 {
		jOp.Payload = json.RawMessage{}
	}
	return json.Marshal(jOp)
}

type jsonOp struct {
	Type       string          `json:"type"`
	OccurredAt ISO8601         `json:"occurred_at"`
	Payload    json.RawMessage `json:"payload,omitempty"`
}

func NewOp(applier Applier, occurredAt time.Time) *Op {
	return &Op{
		Type:       applier.Type(),
		OccurredAt: ISO8601(occurredAt),
		Payload:    applier,
	}
}

var knownAppliers = map[string]func(message json.RawMessage) (Applier, error){}

func (o *Op) UnmarshalJSON(bytes []byte) error {
	var jOp jsonOp
	err := json.Unmarshal(bytes, &jOp)
	if err != nil {
		return err
	}
	o.Type = jOp.Type
	o.OccurredAt = jOp.OccurredAt
	mkApplier, ok := knownAppliers[jOp.Type]
	if !ok {
		return fmt.Errorf("unrecognized operation %s", o.Type)
	}
	o.Payload, err = mkApplier(jOp.Payload)
	if err != nil {
		return err
	}
	return nil
}

func init() {
	knownAppliers["Push"] = func(message json.RawMessage) (Applier, error) {
		var applier PushApplier
		if err := json.Unmarshal(message, &applier); err != nil {
			fmt.Println("in push")
			return nil, err
		}
		return &applier, nil
	}
	knownAppliers["Pop"] = func(message json.RawMessage) (Applier, error) {
		var applier PopApplier
		return &applier, nil
	}
	knownAppliers["Focus"] = func(message json.RawMessage) (Applier, error) {
		var applier FocusApplier
		if err := json.Unmarshal(message, &applier); err != nil {
			fmt.Println("in focus")
			return nil, err
		}
		return &applier, nil
	}
}

type Applier interface {
	Type() string
	Apply([]Item) []Item
	json.Unmarshaler
}

type PushApplier Item

func (p *PushApplier) Type() string {
	return "Push"
}

func (p *PushApplier) Apply(items []Item) []Item {
	return append(items, Item(*p))
}

func (p *PushApplier) UnmarshalJSON(bytes []byte) error {
	var item Item
	err := json.Unmarshal(bytes, &item)
	if err != nil {
		return err
	}
	*p = PushApplier(item)
	return nil
}

type PopApplier struct{}

func (p *PopApplier) MarshalJSON() ([]byte, error) {
	return json.Marshal(nil)
}

func (p *PopApplier) Type() string {
	return "Pop"
}

func (p *PopApplier) Apply(items []Item) []Item {
	if len(items) == 0 {
		return items
	}
	return items[:len(items)-1]
}

func (p *PopApplier) UnmarshalJSON(bytes []byte) error {
	return nil
}

type FocusApplier uint

func (p *FocusApplier) Type() string {
	return "Focus"
}

func (p *FocusApplier) Apply(items []Item) []Item {
	i := *p
	newItems := make([]Item, len(items)-1, len(items))
	copy(newItems[:i], items[:i])
	copy(newItems[i:], items[i+1:])
	return append(newItems, items[i])
}

func (p *FocusApplier) UnmarshalJSON(bytes []byte) error {
	var i uint
	if err := json.Unmarshal(bytes, &i); err != nil {
		return err
	}
	*p = FocusApplier(i)
	return nil
}
