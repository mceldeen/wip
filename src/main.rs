#![feature(termination_trait_lib)]
#![feature(try_trait)]
extern crate chrono;

use std::env;
use std::string::ToString;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use chrono::Local;
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
  let wip = WIP::new(wip_filename);
  if args.len() > 0 {
    let command = args[0].as_str();
    let command_args: Vec<&str> = args.iter().skip(1).map(|a| a.as_ref()).collect();
    match command {
      "push" => {
        let item: Item = Item(command_args.join(" "));
        if !item.is_empty() {
          wip.push(item).map_err(WIPCLIError::IOError)
        } else {
          Err(WIPCLIError::ArgumentError(String::from("nothing to push")))
        }
      }
      "pop" => {
        wip.pop().map_err(WIPCLIError::IOError)
      }
      "focus" => {
        command_args.get(0)
          .into_result()
          .map_or_else(
            |_| Err(
              WIPCLIError::ArgumentError(String::from("no focus index supplied"))
            ),
            |i| i.parse::<u64>().map_err(|_|
              WIPCLIError::ArgumentError(String::from("index is not a number"))
            ),
          )
          .and_then(|index| {
            wip.focus(index).map_err(WIPCLIError::IOError)
          })
      }
      _ => {
        Err(WIPCLIError::ArgumentError(format!("unrecognized command {}", command)))
      }
    }?
  }
  println!("{}", wip.show().map_err(WIPCLIError::IOError)?);
  let wip_timing = env::var("WIP_TIMING")
    .map_or_else(
      |_| false,
      |t| t == String::from("true"),
    );
  if wip_timing {
    let runtime = Instant::now().duration_since(start);
    println!("done in {}µs", runtime.as_micros());
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
    -1
  }
}

fn get_wip_filename() -> Result<PathBuf, VarError> {
  env::var("WIP_FILENAME")
    .map_or_else(
      |_| env::var("HOME").map(
        |home| Path::new(&home).join(".wip")
      ),
      |wip_filename| Ok(Path::new(&wip_filename).to_path_buf()),
    )
}

#[derive(Serialize, Deserialize, Debug)]
struct WIP {
  filename: PathBuf,
}

impl WIP {
  pub fn new(filename: PathBuf) -> WIP {
    WIP { filename }
  }

  pub fn ops(&self) -> io::Result<Vec<Op>> {
    read_lines(self.filename.as_path()).and_then(
      |lines| lines
        .into_iter()
        .map(
          |line| line.and_then(|json|
            serde_json::from_str(json.as_str())
              .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
          )
        )
        .collect()
    )
  }

  pub fn items(&self) -> io::Result<Vec<Item>> {
    self.ops().map(
      |ops| ops
        .into_iter()
        .fold(
          vec![],
          |items, op| op.apply(items),
        )
    )
  }

  pub fn show(&self) -> io::Result<String> {
    self.items().map(|items| {
      if items.len() == 0 {
        String::from("No WIP")
      } else {
        items
          .into_iter()
          .enumerate()
          .rev()
          .fold(
            String::from(""),
            |buf, (i, item)| format!("{}{}: {}\n", buf, i, item.to_string()),
          )
      }
    })
  }

  pub fn push(&self, item: Item) -> io::Result<()> {
    self.write_op(Op {
      occurred_at: Local::now().to_rfc3339(),
      payload: Payload::Push(item),
    })
  }

  pub fn pop(&self) -> io::Result<()> {
    self.write_op(Op {
      occurred_at: Local::now().to_rfc3339(),
      payload: Payload::Pop,
    })
  }

  pub fn focus(&self, index: u64) -> io::Result<()> {
    self.write_op(Op {
      occurred_at: Local::now().to_rfc3339(),
      payload: Payload::Focus(index),
    })
  }

  fn write_op(&self, op: Op) -> io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    OpenOptions::new()
      .write(true)
      .create(true)
      .append(true)
      .open(self.filename.as_path())
      .and_then(|mut file| {
        serde_json::to_string(&op)
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
          .and_then(|json| {
            writeln!(file, "{}", json)
          })
      })
  }
}

fn read_lines<P>(
  filename: P
) -> io::Result<io::Lines<io::BufReader<File>>>
  where P: AsRef<Path>,
{
  File::open(filename)
    .map(|file| io::BufReader::new(file).lines())
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
  pub occurred_at: String,

  #[serde(flatten)]
  pub payload: Payload,
}

impl Op {
  fn apply(self, items: Vec<Item>) -> Vec<Item> {
    self.payload.apply(items)
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
  fn apply(self, mut items: Vec<Item>) -> Vec<Item> {
    match self {
      Payload::Push(item) => {
        items.push(item);
      }
      Payload::Pop => {
        items.pop();
      }
      Payload::Focus(index) => {
        let removed = items.remove(index as usize);
        items.push(removed);
      }
    }
    items
  }
}
