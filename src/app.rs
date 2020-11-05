use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct FileList {
    files: Vec<FileInfo>,
    selected_file: Option<usize>,
}
impl FileList {
    fn get_selected_file(&self) -> Option<&FileInfo> {
        match self.selected_file {
            None => None,
            Some(i) => self.files.get(i),
        }
    }

    fn unselect_file(&mut self) {
        self.selected_file = None
    }

    fn select_first(&mut self) {
        if self.files.len() > 0 {
            self.selected_file = Some(0);
        }
    }

    /// Select next file in list, returning whether the file selection has changed.
    fn select_next(&mut self) -> bool {
        let prev = self.selected_file;
        if let Some(selected) = self.selected_file {
            self.selected_file =
                Some((selected + 1).min(self.files.len().saturating_sub(1)));
        }
        if prev != self.selected_file {
            true
        } else {
            false
        }
    }

    /// Select previous file in list, returning whether the file selection has changed.
    fn select_prev(&mut self) -> bool {
        let prev = self.selected_file;
        if let Some(selected) = self.selected_file {
            self.selected_file = Some(selected.saturating_sub(1));
        }
        if prev != self.selected_file {
            true
        } else {
            false
        }
    }
}
impl Default for FileList {
    fn default() -> Self {
        FileList {
            files: vec![],
            selected_file: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    name: String,
    path: PathBuf,
}

impl FileInfo {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn is_dir(&self) -> bool {
        self.path.is_dir()
    }
}

impl From<fs::DirEntry> for FileInfo {
    fn from(dir_entry: fs::DirEntry) -> Self {
        let mut name = dir_entry.file_name().to_string_lossy().to_string();
        if dir_entry.path().is_dir() {
            name.push('/');
        }
        FileInfo {
            name,
            path: dir_entry.path(),
        }
    }
}

// TODO: define `current` and `selected`
#[derive(Debug, Clone)]
pub struct State {
    base_path: std::path::PathBuf,
    flists: [FileList; 2],
    selected_flist: usize,
}

// FIXME: don't allow state of selecting a list with no files in it
impl State {
    pub fn new() -> Result<State, std::io::Error> {
        let init_path = std::env::current_dir()?;
        let current_dir_files = State::read_file_list(&init_path);
        let selected_file = current_dir_files.first().map(|_| 0);

        let flists = [
            FileList {
                files: current_dir_files,
                selected_file,
            },
            FileList::default(),
        ];

        let mut state = State {
            base_path: init_path,
            flists,
            selected_flist: 0,
        };

        state.flists[1] = FileList {
            files: state.files_in_selected_dir(),
            selected_file: None,
        };

        Ok(state)
    }

    pub fn base_path(&self) -> String {
        self.base_path.to_string_lossy().to_string()
    }

    pub fn on_up(&mut self) {
        if self.current_list_mut().select_prev() {
            self.refresh_files_to_right();
        }
    }

    pub fn on_down(&mut self) {
        if self.current_list_mut().select_next() {
            self.refresh_files_to_right();
        }
    }

    pub fn on_left(&mut self) {
        if self.selected_flist != 0 {
            self.current_list_mut().unselect_file();
            self.selected_flist -= 1;
        } else if let Some(parent) = self.base_path.parent() {
            let parent = parent.to_owned();

            self.current_list_mut().unselect_file();

            let files = State::read_file_list(&parent);
            let selected_file =
                files.binary_search_by_key(&self.base_path, |f| f.path.clone());
            self.flists = [
                FileList {
                    files,
                    // FIXME: I bet we can somehow go into a parent directory with no visible files...
                    selected_file: selected_file.ok(),
                },
                self.flists[0].clone(),
            ];

            self.base_path = parent
        }
    }

    pub fn on_right(&mut self) {
        if self.selected_file_is_dir() {
            if self.space_to_right() {
                self.selected_flist += 1;
                self.current_list_mut().select_first();
            } else if let Some(selected_dir_path) = self.selected_file_path() {
                let selected_dir_path = selected_dir_path.clone();

                let files = State::read_file_list(&selected_dir_path);
                let selected_file = files.first().map(|_| 0);
                self.flists = [
                    self.flists[1].clone(),
                    FileList {
                        files,
                        selected_file,
                    },
                ];

                // FIXME: no unwrap!
                self.base_path = selected_dir_path.parent().unwrap().to_owned();
            }
        }
    }

    pub fn selected_file_in_list(&self, list: usize) -> Option<usize> {
        self.flists.get(list).and_then(|l| l.selected_file)
    }

    pub fn files_in_list(&self, list: usize) -> Option<&Vec<FileInfo>> {
        self.flists.get(list).map(|l| &l.files)
    }

    fn files_in_selected_dir(&self) -> Vec<FileInfo> {
        match self.current_list().get_selected_file() {
            None => Vec::default(),
            Some(file) => {
                if file.is_dir() {
                    State::read_file_list(&file.path)
                } else {
                    Vec::default()
                }
            }
        }
    }

    fn read_file_list<P: AsRef<Path>>(path: &P) -> Vec<FileInfo> {
        let mut files = path.as_ref().read_dir().map_or(vec![], |contents| {
            contents
                .filter_map(Result::ok)
                .map(|entry| FileInfo::from(entry))
                .collect()
        });
        files.sort_by(|f1, f2| f1.name.cmp(&f2.name));
        files
    }

    fn space_to_right(&self) -> bool {
        self.selected_flist < (self.flists.len() - 1)
    }

    fn refresh_files_to_right(&mut self) {
        if self.space_to_right() {
            self.flists[self.selected_flist + 1] = FileList {
                files: self.files_in_selected_dir(),
                selected_file: None,
            };
        }
    }

    fn selected_file_is_dir(&self) -> bool {
        self.current_list()
            .get_selected_file()
            .map_or(false, |f| f.is_dir())
    }

    fn selected_file_path(&self) -> Option<&PathBuf> {
        self.current_list().get_selected_file().map(|f| &f.path)
    }

    fn current_list(&self) -> &FileList {
        &self.flists[self.selected_flist]
    }

    fn current_list_mut(&mut self) -> &mut FileList {
        &mut self.flists[self.selected_flist]
    }
}
