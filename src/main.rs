mod terminal;

use terminal::Terminal;

fn main() {
    {
        let mut term = Terminal::new();
        term.update();
    }

    println!("Test");
}
