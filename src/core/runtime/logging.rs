use derive_more::Display;
use std::io::Write;
use std::sync::{LazyLock, Mutex};

#[derive(PartialEq, Display)]
pub enum Message {
    Info(String),
    Debug(String),
    Warn(String),
    Error(String),
}

impl std::process::Termination for Message {
    fn report(self) -> std::process::ExitCode {
        if matches!(self, Message::Error(_)) {
            return 1.into()
        }
        0.into()
    }
}

impl Message {
    pub fn info(msg: &str) -> Message {
        Message::Info(msg.into())
    }
    pub fn debug(msg: &str) -> Message {
        Message::Debug(msg.into())
    }
    pub fn warn(msg: &str) -> Message {
        Message::Warn(msg.into())
    }
    pub fn error(msg: &str) -> Message {
        Message::Error(msg.into())
    }
    pub fn inner(&self) -> String {
        self.to_string()
    }
}

pub struct Logging {
    msg_queue: Vec<Message>,
}

impl Logging {
    pub fn init() -> Logging {
        Self {
            msg_queue: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.msg_queue.clear();
    }

    pub fn info(&mut self, msg: &str) {
        self.msg_queue.push(Message::info(msg))
    }

    pub fn debug(&mut self, msg: &str) {
        self.msg_queue.push(Message::debug(msg))
    }

    pub fn warn(&mut self, msg: &str) {
        self.msg_queue.push(Message::warn(msg))
    }

    pub fn error(&mut self, msg: &str) {
        self.msg_queue.push(Message::error(msg))
    }

    pub fn process(&self) {
        if !self.msg_queue.is_empty() {
            for msg in self.msg_queue.iter() {
                print!("\x1b[38;2;200;150;255mLOG: ");
                match msg {
                    Message::Info(_) => print!("\x1b[38;2;100;150;255m"),
                    Message::Debug(_) => print!("\x1b[38;2;255;200;50m"),
                    Message::Warn(_) => print!("\x1b[38;2;255;100;50m"),
                    Message::Error(_) => print!("\x1b[38;2;255;50;50m")
                }
                print!("{}", msg);
                print!("\x1b[0m");
            }
            println!();
        }
    }

    pub fn to_file(&self) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::create("log.txt")?;
        let mut buf = vec![0u8; 65536];

        for msg in self.msg_queue.iter().as_ref() {
            buf.append(&mut format!("{}\n", msg.inner()).as_bytes().to_vec())
        }
        _ = file.write_all(&buf);
        Ok(())
    }
}

pub static LOGGER: LazyLock<Mutex<Logging>> = LazyLock::new(|| Mutex::new(Logging::init()));

pub fn info(msg: &str)  { LOGGER.lock().unwrap().info(msg) }
pub fn debug(msg: &str) { LOGGER.lock().unwrap().debug(msg) }
pub fn warn(msg: &str)  { LOGGER.lock().unwrap().warn(msg) }
pub fn error(msg: &str) { LOGGER.lock().unwrap().error(msg) }
pub fn process()        { LOGGER.lock().unwrap().process() }
pub fn to_file()        { LOGGER.lock().unwrap().to_file().unwrap() }