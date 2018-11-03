extern crate termion;

mod buffer;
mod cursor;
mod interface;

use termion::color;
use termion::style;

pub fn nep() {
    let interface = interface::Interface::from(std::io::stdout());
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
