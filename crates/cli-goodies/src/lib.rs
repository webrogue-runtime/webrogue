use crossterm::{
    style::{Color, Stylize as _},
    terminal::ClearType,
};
use std::{
    io::{stderr, stdin, stdout, IsTerminal, Write},
    sync::atomic::{AtomicBool, Ordering},
};
mod select;
mod step;

pub use inquire;
pub use select::select;
pub use step::{step, step_async};

pub fn write_error(err: &str) {
    if is_tty() {
        let _ = crossterm::execute!(
            stderr(),
            crossterm::style::PrintStyledContent(crossterm::style::StyledContent::new(
                crossterm::style::ContentStyle::new().red(),
                err
            ))
        );
        let _ = crossterm::execute!(stderr(), crossterm::cursor::MoveToNextLine(1));
    } else {
        eprintln!("{}", err);
    }
}

pub fn warning(message: &str) {
    print_message("warning", message, Color::Red);
}
pub fn note(message: &str) {
    print_message("note", message, Color::Cyan);
}

fn print_message(label: &str, message: &str, color: Color) {
    if is_tty() {
        let mut style = crossterm::style::ContentStyle::new();
        style.foreground_color = Some(color);
        let _ = crossterm::queue!(stderr(), crossterm::terminal::Clear(ClearType::CurrentLine));
        let _ = crossterm::queue!(stderr(), crossterm::cursor::MoveToColumn(0));
        let _ = crossterm::queue!(
            stderr(),
            crossterm::style::PrintStyledContent(crossterm::style::StyledContent::new(
                style, label
            ))
        );
        let _ = crossterm::queue!(
            stderr(),
            crossterm::style::Print(format!(": {}\n", message))
        );
        let _ = stderr().flush();
    } else {
        if step::is_step() && !step::is_dirty() {
            eprintln!("");
        }
        eprintln!("{}: {}", label, message);
    }
    step::set_dirty();
}

pub fn disable_raw_mode() {
    let _ = crossterm::terminal::disable_raw_mode();
}

static IS_INITED: AtomicBool = AtomicBool::new(false);

fn check_init() {
    if IS_INITED.fetch_or(true, Ordering::SeqCst) {
        return;
    }
    // TODO use update_hook when it is standardized
    // https://github.com/rust-lang/rust/issues/92649
    let old_handler = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        disable_raw_mode();
        old_handler(info)
    }));
}

pub fn is_tty() -> bool {
    stdin().is_terminal() && stdout().is_terminal() && stderr().is_terminal()
}
