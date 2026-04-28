use std::{
    future::Future,
    io::{stdout, Write},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use spinners::{Spinner, Spinners};

static IS_DIRTY: AtomicBool = AtomicBool::new(false);
static IS_STEP: AtomicBool = AtomicBool::new(true);

struct StepState {
    message: String,
    start_time: Instant,
    spinner: Option<Spinner>,
}

pub fn step<T, E>(message: String, func: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    let state = step_start(message);
    let result = func();
    step_stop(state, result.is_ok());
    result
}

pub async fn step_async<T, E, Fut: Future<Output = Result<T, E>>>(
    message: String,
    func: impl FnOnce() -> Fut,
) -> Result<T, E> {
    let state = step_start(message);
    let result = func().await;
    step_stop(state, result.is_ok());
    result
}

fn step_start(message: String) -> StepState {
    let start_time = Instant::now();
    let spinner = if crate::is_tty() {
        // let _ = crossterm::execute!(stdout(), crossterm::cursor::Hide);
        Some(Spinner::new(Spinners::Dots10, format!("{}...", message)))
    } else {
        print!("{}...", message);
        let _ = stdout().flush();
        None
    };
    IS_STEP.store(true, Ordering::SeqCst);
    IS_DIRTY.store(false, Ordering::SeqCst);
    StepState {
        message,
        spinner,
        start_time,
    }
}

fn step_stop(state: StepState, is_ok: bool) {
    let time = state.start_time.elapsed().as_secs_f64();
    let time = if time < 5.0 {
        format!("{:.1}", time)
    } else {
        format!("{:.0}", time)
    };
    let time = format!("{} in {}s", if is_ok { "done" } else { "failed" }, time);
    IS_STEP.store(false, Ordering::SeqCst);
    let is_dirty = IS_DIRTY.fetch_and(false, Ordering::SeqCst);
    crate::check_init();
    if let Some(mut spinner) = state.spinner {
        spinner.stop_and_persist(
            if is_ok { "✓" } else { "✗" },
            format!("{}...{}", state.message, time),
        );
    } else {
        if is_dirty {
            print!("{}...", state.message);
        }
        println!("{}", time);
    };
    crate::disable_raw_mode();
}

pub(crate) fn set_dirty() {
    IS_DIRTY.fetch_or(true, Ordering::SeqCst);
}
pub(crate) fn is_dirty() -> bool {
    IS_DIRTY.load(Ordering::SeqCst)
}
pub(crate) fn is_step() -> bool {
    IS_STEP.load(Ordering::SeqCst)
}
