extern crate chrono;

use self::chrono::Local;

pub struct Logger(String);

impl Logger {
    pub fn with_tag(tag: &str) -> Logger {
        Logger(tag.to_owned())
    }

    fn log(&self, lvl: &str, msg: &str) {
        println!(
            "[{}][{}][{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            lvl,
            self.0,
            msg
        );
    }

    pub fn info(&self, msg: &str) {
        self.log("INF", msg);
    }

    #[allow(dead_code)]
    pub fn warning(&self, msg: &str) {
        self.log("WRN", msg);
    }

    pub fn error(&self, msg: &str) {
        self.log("ERR", msg);
    }
}