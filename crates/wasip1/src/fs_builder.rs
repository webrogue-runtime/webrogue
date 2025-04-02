pub struct Dir {
    pub dirs: std::collections::BTreeMap<String, Dir>,
    pub files: std::collections::BTreeMap<String, File>,
}

impl Dir {
    pub fn root(wrapp: webrogue_wrapp::WrappHandle) -> Self {
        let mut result = Self::empty();
        for (path, position) in wrapp.file_index().file_positions {
            let path_parts = path
                .split("/")
                .filter(|path_part| !path_part.is_empty())
                .collect::<Vec<_>>();
            let last_path_part = path_parts.last().unwrap();
            result.insert_file_position(
                last_path_part,
                position,
                &path_parts.as_slice()[0..(path_parts.len() - 1)],
                &wrapp,
            );
        }
        result
    }

    fn empty() -> Self {
        Self {
            dirs: std::collections::BTreeMap::new(),
            files: std::collections::BTreeMap::new(),
        }
    }

    fn insert_file_position(
        &mut self,
        filename: &str,
        position: webrogue_wrapp::file_index::FilePosition,
        parts: &[&str],
        wrapp: &webrogue_wrapp::WrappHandle,
    ) {
        if parts.is_empty() {
            self.files
                .insert(filename.to_owned(), File::new(wrapp.clone(), position));
        } else {
            let part = parts.first().unwrap();
            if !self.dirs.contains_key(*part) {
                self.dirs.insert(part.to_owned().to_owned(), Dir::empty());
            }
            self.dirs.get_mut(*part).unwrap().insert_file_position(
                filename,
                position,
                &parts[1..],
                wrapp,
            );
        }
    }
}

pub struct File {
    pub wrapp: webrogue_wrapp::WrappHandle,
    pub position: webrogue_wrapp::file_index::FilePosition,
}

impl File {
    pub fn new(
        wrapp: webrogue_wrapp::WrappHandle,
        position: webrogue_wrapp::file_index::FilePosition,
    ) -> Self {
        Self { wrapp, position }
    }
}
