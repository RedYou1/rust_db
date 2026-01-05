#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BDPath {
    pub dir_path: String,
    pub rel_file_path: String,
}

impl BDPath {
    pub fn new_main_str(path: &str) -> Self {
        Self {
            dir_path: path.to_owned(),
            rel_file_path: "main.bin".to_owned(),
        }
    }

    pub fn new_main(path: String) -> Self {
        Self {
            dir_path: path,
            rel_file_path: "main.bin".to_owned(),
        }
    }

    pub fn new_dyn(path: String, nb: usize) -> Self {
        Self {
            dir_path: path,
            rel_file_path: format!("dyn/{nb}.bin"),
        }
    }

    pub fn new_index(path: String, name: String) -> Self {
        Self {
            dir_path: path,
            rel_file_path: format!("index/{name}.bin"),
        }
    }

    pub fn full(&self) -> String {
        format!("{}/{}", self.dir_path, self.rel_file_path)
    }

    pub fn dyn_path(&self) -> String {
        format!("{}/dyn", self.dir_path)
    }

    pub fn folder(&self) -> String {
        format!(
            "{}{}",
            self.dir_path,
            self.rel_file_path
                .split_once("/")
                .map_or("".to_owned(), |f| format!("/{}", f.0))
        )
    }

    pub fn index_path(&self) -> String {
        format!("{}/index", self.dir_path)
    }
}
