use std::fmt::Display;

pub fn select<T: Display>(
    message: &str,
    options: Vec<T>,
    initial_index: Option<usize>,
) -> anyhow::Result<(T, usize)> {
    anyhow::ensure!(
        crate::is_tty(),
        "Can't interactively select an item on a non-TTY stdin/stdout/stderr"
    );
    crate::check_init();
    let mut select = inquire::Select::new(message, options);
    if let Some(index) = initial_index {
        select = select.with_starting_cursor(index);
    }
    let result = select.raw_prompt();
    crate::disable_raw_mode();
    let result = result?;
    Ok((result.value, result.index))
}
