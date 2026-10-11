pub fn is_app_id_valid(id: &str) -> bool {
    let parts = id.split('.').collect::<Vec<_>>();

    parts.len() >= 2
        && parts.iter().all(|part| {
            part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                && part.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        })
}

pub fn is_app_name_valid(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_alphanumeric() || ".-_+ ".contains(c))
}

#[cfg(test)]
mod tests {
    use super::{is_app_id_valid, is_app_name_valid};

    #[test]
    fn validates_app_ids() {
        assert!(is_app_id_valid("com.something.something-else"));

        assert!(!is_app_id_valid("com"));
        assert!(!is_app_id_valid("9om.something"));
        assert!(!is_app_id_valid("com..something"));
    }

    #[test]
    fn validates_app_names() {
        assert!(is_app_name_valid("My App+ 2.0_é"));
        assert!(!is_app_name_valid("name/with/slashes"));
    }
}
