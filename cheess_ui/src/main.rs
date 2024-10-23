mod gui;
use gui::run;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
    }
}