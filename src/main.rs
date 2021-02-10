#![feature(termination_trait_lib)]
#![feature(try_trait)]
extern crate chrono;

use std::env;
use std::string::ToString;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};
use std::time::Instant;
use std::env::VarError;
use std::result::Result;
use std::process::Termination;
use std::fmt;
use std::ops::Try;

fn main() -> Result<(), WIPCLIError> {
    let start = Instant::now();
    let args: Vec<String> = env::args().skip(1).collect();
    let wip_filename = get_wip_filename().map_err(|_| WIPCLIError::WIPFilenameNotFound)?;
    let mut wip = WIP::new(wip_filename).map_err(|e| WIPCLIError::IOError(e))?;
    if args.len() > 0 {
        let command = args[0].as_str();
        let command_args: Vec<&str> = args.iter().skip(1).map(|a| a.as_ref()).collect();
        match command {
            "push" => {
                let item: Item = Item(command_args.join(" "));
                if !item.is_empty() {
                    wip.push(item).map_err(|e| WIPCLIError::IOError(e))
                } else {
                    Err(WIPCLIError::ArgumentError(String::from("nothing to push")))
                }
            }
            "pop" => {
                wip.pop().map_err(|e| WIPCLIError::IOError(e))
            }
            "focus" => {
                let index: u64 = command_args.get(0)
                    .into_result()
                    .map_or_else(
                        |_| Err(
                            WIPCLIError::ArgumentError(String::from("no focus index supplied"))
                        ),
                        |i| i.parse::<u64>().map_err(|_|
                            WIPCLIError::ArgumentError(String::from("index is not a number"))
                        ),
                    )?;
                wip.focus(index).map_err(|e| WIPCLIError::IOError(e))
            }
            _ => {
                Err(WIPCLIError::ArgumentError(format!("unrecognized command {}", command)))
            }
        }?
    }
    println!("{}", wip.show());
    let show_timing = env::var("WIP_TIMING").map(|t| t == String::from("true")).unwrap_or(false);
    if show_timing {
        println!("done in {}µs", Instant::now().duration_since(start).as_micros());
    }
    Ok(())
}

enum WIPCLIError {
    WIPFilenameNotFound,
    IOError(io::Error),
    ArgumentError(String),
}

impl fmt::Debug for WIPCLIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WIPCLIError::WIPFilenameNotFound => {
                write!(f, "if WIP_FILENAME was not specified so it defaulted to $HOME/.wip but HOME was not defined")
            }
            WIPCLIError::IOError(e) => {
                write!(f, "io error: {}", e)
            }
            WIPCLIError::ArgumentError(e) => {
                write!(f, "argument error: {}", e)
            }
        }
    }
}

impl Termination for WIPCLIError {
    fn report(self) -> i32 {
        use WIPCLIError::*;
        match self {
            WIPFilenameNotFound => -1,
            IOError(_) => -2,
            ArgumentError(_) => -3,
        }
    }
}

fn get_wip_filename() -> Result<PathBuf, VarError> {
    env::var("WIP_FILENAME")
        .map(|wip_filename| {
            Path::new(&wip_filename).to_path_buf()
        })
        .or_else(|_| {
            env::var("HOME").map(|home| {
                Path::new(&home).join(".wip")
            })
        })
}

#[derive(Serialize, Deserialize, Debug)]
struct WIP {
    filename: PathBuf,
    ops: Vec<Op>,
}

impl WIP {
    pub fn new(filename: PathBuf) -> io::Result<WIP> {
        let mut ops = vec![];
        if let Ok(lines) = read_lines(filename.as_path()) {
            for line in lines {
                ops.push(serde_json::from_str(line?.as_str())?);
            }
        }
        Ok(WIP { filename, ops })
    }

    pub fn items(&self) -> Vec<Item> {
        let mut items = vec![];
        for op in self.ops.iter() {
            op.apply(&mut items);
        }
        items
    }

    pub fn show(&self) -> String {
        let items = self.items();
        if items.len() == 0 {
            return String::from("No WIP");
        }
        let mut buf = String::from("");
        for (i, item) in items.iter().enumerate().rev() {
            buf += format!("{}: {}\n", i, item.to_string()).as_ref();
        }
        buf
    }

    pub fn push(&mut self, item: Item) -> io::Result<()> {
        self.write_op(Op {
            occurred_at: Local::now(),
            payload: Payload::Push(item),
        })
    }

    pub fn pop(&mut self) -> io::Result<()> {
        self.write_op(Op {
            occurred_at: Local::now(),
            payload: Payload::Pop,
        })
    }

    pub fn focus(&mut self, index: u64) -> io::Result<()> {
        self.write_op(Op {
            occurred_at: Local::now(),
            payload: Payload::Focus(index),
        })
    }

    fn write_op(&mut self, op: Op) -> io::Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(self.filename.as_path())?;
        writeln!(file, "{}", serde_json::to_string(&op).unwrap().as_str())?;
        self.ops.push(op);
        Ok(())
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Item(String);

impl ToString for Item {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Item {
    fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Op {
    #[serde(with = "iso8601")]
    pub occurred_at: DateTime<Local>,

    #[serde(flatten)]
    pub payload: Payload,
}

impl Op {
    fn apply(&self, items: &mut Vec<Item>) -> Option<Item> {
        self.payload.apply(items)
    }
}

mod iso8601 {
    use chrono::{DateTime, TimeZone, Local};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%FT%T%z";

    pub fn serialize<S>(
        date: &DateTime<Local>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Local.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
enum Payload {
    Push(Item),
    Pop,
    Focus(u64),
}

impl Payload {
    fn apply(&self, items: &mut Vec<Item>) -> Option<Item> {
        match self {
            &Payload::Push(ref item) => {
                items.push(item.clone());
                Some(item.clone())
            }
            &Payload::Pop => {
                items.pop()
            }
            &Payload::Focus(index) => {
                let removed = items.remove(index as usize);
                items.push(removed.clone());
                Some(removed)
            }
        }
    }
}