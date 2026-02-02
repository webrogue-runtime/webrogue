use webrogue_wrapp::IVFSHandle;

pub struct Dir<VFSHandle: webrogue_wrapp::IVFSHandle> {
    pub dirs: std::collections::BTreeMap<String, Dir<VFSHandle>>,
    pub files: std::collections::BTreeMap<String, File<VFSHandle>>,
}

impl<VFSHandle: webrogue_wrapp::IVFSHandle> Dir<VFSHandle> {
    pub fn root(handle: VFSHandle) -> Self {
        let mut result = Self::empty();
        for (path, position) in handle.get_index() {
            let path_parts = path
                .split("/")
                .filter(|path_part| !path_part.is_empty())
                .collect::<Vec<_>>();
            let last_path_part = path_parts.last().unwrap();
            result.insert_file_position(
                last_path_part,
                position.clone(),
                &path_parts.as_slice()[0..(path_parts.len() - 1)],
                &handle,
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
        position: <VFSHandle as IVFSHandle>::FilePosition,
        parts: &[&str],
        wrapp: &VFSHandle,
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

pub struct File<VFSHandle: webrogue_wrapp::IVFSHandle> {
    pub handle: VFSHandle,
    pub position: VFSHandle::FilePosition,
}

impl<VFSHandle: webrogue_wrapp::IVFSHandle> File<VFSHandle> {
    pub fn new(handle: VFSHandle, position: VFSHandle::FilePosition) -> Self {
        Self { handle, position }
    }
}
