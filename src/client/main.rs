mod terminal;

use terminal::Terminal;

fn main() {
    {
        let mut term = Terminal::new();
        // Initialisation and setup would be done here
        term.start();
    }

    println!("nep exited :((");
    println!("hope you loved it <3");
}
