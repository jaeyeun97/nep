extern crate dbus;

use dbus::{BusType, Connection, Message};

mod terminal;

use terminal::Terminal;

fn main() {
    let c = Connection::get_private(BusType::Session).unwrap();
    let m = Message::new_method_call("org.nep", "/hello", "org.nep", "Hello").unwrap();
    c.send(m).unwrap();

    {
        let mut term = Terminal::new();
        // Initialisation and setup would be done here
        term.start();
    }

    println!("nep exited :((");
    println!("hope you loved it <3");
}
