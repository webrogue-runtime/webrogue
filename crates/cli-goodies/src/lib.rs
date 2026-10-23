use crossterm::{
    style::{ContentStyle, Stylize as _},
    terminal::ClearType,
};
use std::{
    io::{IsTerminal, Write, stderr, stdin, stdout},
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
            stderr().lock(),
            crossterm::style::PrintStyledContent(crossterm::style::StyledContent::new(
                crossterm::style::ContentStyle::new().red(),
                err,
            )),
            crossterm::cursor::MoveToNextLine(1)
        );
    } else {
        eprintln!("{}", err);
    }
}
pub fn error(message: &str) {
    print_message("error", message, ContentStyle::new().red().bold());
}
pub fn warning(message: &str) {
    print_message("warning", message, ContentStyle::new().yellow().bold());
}
pub fn note(message: &str) {
    print_message("note", message, ContentStyle::new().cyan());
}

fn print_message(label: &str, message: &str, label_style: ContentStyle) {
    let mut stderr = stderr().lock();
    if is_tty() {
        let _ = crossterm::execute!(
            stderr,
            crossterm::terminal::Clear(ClearType::CurrentLine),
            crossterm::cursor::MoveToColumn(0),
            crossterm::style::PrintStyledContent(crossterm::style::StyledContent::new(
                label_style,
                label
            )),
            crossterm::style::Print(format!(": {}\n", message))
        );
    } else {
        if step::is_step() && !step::is_dirty() {
            let _ = writeln!(stderr, "");
        }
        let _ = writeln!(stderr, "{}: {}", label, message);
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
