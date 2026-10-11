use std::cmp::min;

pub fn get_safe_range<T>(data: &[T], start: usize, len: usize) -> &[T] {
    let data = &data[min(data.len(), start)..];
    &data[..min(data.len(), len)]
}

#[cfg(test)]
mod tests {
    use super::get_safe_range;

    #[test]
    fn clamps_start_and_length() {
        let data = [10, 20, 30, 40];

        assert_eq!(get_safe_range(&data, 1, 2), &[20, 30]);
        assert_eq!(get_safe_range(&data, 2, usize::MAX), &[30, 40]);
        assert_eq!(get_safe_range(&data, data.len(), 1), &[]);
        assert_eq!(get_safe_range(&data, usize::MAX, 1), &[]);
        assert_eq!(get_safe_range(&data, 1, 0), &[]);
    }
}
