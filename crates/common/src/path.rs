pub fn is_valid_filename(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ')
        && !name.contains("..")
}

pub fn split_path(path: &str, min_level: u32) -> Option<Vec<&str>> {
    let mut result = Vec::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if result.len() > min_level as usize {
                result.pop();
                continue;
            } else {
                return None;
            }
        }
        if !is_valid_filename(part) {
            return None;
        }
        result.push(part);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::{is_valid_filename, split_path};

    #[test]
    fn validates_filenames() {
        assert!(is_valid_filename("my app-2.0_file"));
        assert!(!is_valid_filename("../secret"));
        assert!(!is_valid_filename("name\\with\\slashes"));
        assert!(!is_valid_filename("名前"));
    }

    #[test]
    fn splits_and_normalizes_paths() {
        assert_eq!(
            split_path("/////pt1/.//pt2/remove_this/../pt3", 2)
                .unwrap()
                .join("/"),
            "pt1/pt2/pt3"
        );

        assert_eq!(split_path("/a", 2).unwrap().join("/"), "a");
        assert_eq!(split_path("/root/..", 1), None);
        assert_eq!(split_path("/仅支持ASCII", 1), None);
    }
}
