extern crate termion;

mod buffer;
mod cursor;
mod interface;

use std::sync::{Arc, Mutex};

use termion::color;
use termion::style;

pub fn nep(args: Vec<String>) {
    let interface = if args.len() == 0 {
        interface::Interface::from(std::io::stdout())
    } else {
        let buffer = Arc::new(Mutex::new(buffer::Buffer::from(args[0].clone())));
        interface::Interface::from_buffer(std::io::stdout(), buffer)
    };

    interface.start(std::io::stdin());

    println!("{}nep{} exited", style::Bold, style::Reset);
    println!(
        "hope you loved it {}{}<3{}{}",
        style::Bold,
        color::Fg(color::Red),
        color::Fg(color::Reset),
        style::Reset
    );
}
