mod app;
mod buffer;
mod clipboard;
mod cursor;
mod editor;
mod file_io;
mod highlight;
mod history;
mod input;
mod mode;
mod renderer;
mod status;

use anyhow::Result;
use app::App;
use std::env;

fn print_help() {
    println!(
        "{} {} - minimal terminal text editor\n",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("USAGE:");
    println!("  scribble <filename>\n");
    println!("OPTIONS:");
    println!("  -h, --help       Show this help message");
    println!("  -v, --version    Show version\n");
    println!("EDITOR CONTROLS:");
    println!("  SPACE        Enter insert mode");
    println!("  ESC          Exit insert mode (or ESC)");
    println!("  h j k l      Move cursor");
    println!("  hh           Toggle highlight");
    println!("  c            Copy");
    println!("  v            Paste");
    println!("  x            Cut");
    println!("  u            Undo");
    println!("  r            Redo");
    println!("  s            Save");
    println!("  q            Quit");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
    }

    // If no file provided, open default
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "untitled.txt"
    };

    let mut app = App::new(filename)?;
    app.run()?;

    Ok(())
}
