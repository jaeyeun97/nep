mod terminal;

use terminal::Terminal;

fn main() {
    {
        let mut term = Terminal::new();
        term.start();
    }

    println!("nep exited :((");
    println!("hope you loved it <3");
}
