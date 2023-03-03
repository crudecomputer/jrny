//! Basic implementation of a Log, as none of the complexity of
//! common crates is particularly necessary here.
//! See: <https://docs.rs/log/0.4.11/log/#implementing-a-logger>
use std::io::Write;

use log::{Level, Log, Metadata, Record};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if record.metadata().level() == Level::Warn {
            let mut stderr = StandardStream::stderr(ColorChoice::Always);

            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                .unwrap();
            write!(&mut stderr, "{}", record.args()).unwrap();

            stderr.set_color(ColorSpec::new().set_fg(None)).unwrap();
            writeln!(&mut stderr).unwrap();

            return;
        }

        println!("{}", record.args());
    }

    fn flush(&self) {}
}
