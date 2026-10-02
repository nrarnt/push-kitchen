use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

use bevy::prelude::*;

/// Which kitchens have been solved. Saved between runs of the game.
#[derive(Resource, Debug, Default, PartialEq)]
pub struct Progress {
    /// Level ids. A `BTreeSet` keeps them sorted, so the saved text comes out
    /// the same whatever order the levels were solved in.
    solved: BTreeSet<String>,
}

impl Progress {
    pub fn is_solved(&self, id: &str) -> bool {
        self.solved.contains(id)
    }

    /// Remembers the level as solved. Returns true if it was not already.
    pub fn mark_solved(&mut self, id: &str) -> bool {
        self.solved.insert(id.to_string())
    }

    /// Reads saved progress: one level id per line. Nothing saved yet means
    /// nothing solved yet.
    pub fn load(slot: &SaveSlot) -> io::Result<Progress> {
        let text = slot.read()?;
        let solved = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();
        Ok(Progress { solved })
    }

    pub fn save(&self, slot: &SaveSlot) -> io::Result<()> {
        let lines: Vec<&str> = self.solved.iter().map(String::as_str).collect();
        slot.write(&lines.join("\n"))
    }
}

/// Where progress is kept between runs of the game.
#[derive(Resource, Debug, Clone)]
pub enum SaveSlot {
    /// A text file: on a computer, and in the tests.
    // The web version never makes one of these.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    File(PathBuf),
    /// The storage a browser keeps for this web page.
    #[cfg(target_arch = "wasm32")]
    Browser,
}

impl SaveSlot {
    /// The save slot of the real game: a file in the place macOS gives each
    /// app for its own data, or the browser's storage on the web.
    pub fn for_this_platform() -> SaveSlot {
        #[cfg(target_arch = "wasm32")]
        return SaveSlot::Browser;

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Without a home folder, the file goes next to where the game runs.
            let folder = match std::env::var_os("HOME") {
                Some(home) => PathBuf::from(home).join("Library/Application Support/Push Kitchen"),
                None => PathBuf::new(),
            };
            SaveSlot::File(folder.join("progress.txt"))
        }
    }

    /// The saved text, or an empty string if nothing was saved yet.
    fn read(&self) -> io::Result<String> {
        match self {
            SaveSlot::File(file) => match std::fs::read_to_string(file) {
                Ok(text) => Ok(text),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
                Err(error) => Err(error),
            },
            #[cfg(target_arch = "wasm32")]
            SaveSlot::Browser => browser::read(),
        }
    }

    fn write(&self, text: &str) -> io::Result<()> {
        match self {
            SaveSlot::File(file) => {
                if let Some(folder) = file.parent() {
                    std::fs::create_dir_all(folder)?;
                }
                std::fs::write(file, text)
            }
            #[cfg(target_arch = "wasm32")]
            SaveSlot::Browser => browser::write(text),
        }
    }
}

/// Saving in a web page: the browser's `localStorage`, a small set of named
/// texts it keeps for each website.
#[cfg(target_arch = "wasm32")]
mod browser {
    use std::io;

    const KEY: &str = "push-kitchen-progress";

    fn storage() -> io::Result<web_sys::Storage> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .ok_or_else(|| io::Error::other("this browser gives the page no storage"))
    }

    pub fn read() -> io::Result<String> {
        let saved = storage()?
            .get_item(KEY)
            .map_err(|_| io::Error::other("the browser would not read the saved progress"))?;
        Ok(saved.unwrap_or_default())
    }

    pub fn write(text: &str) -> io::Result<()> {
        storage()?
            .set_item(KEY, text)
            .map_err(|_| io::Error::other("the browser would not save the progress"))
    }
}

/// A save slot that belongs to one test only: a file in the system's temp
/// folder that does not exist yet when the test starts.
#[cfg(test)]
pub fn scratch_slot(test: &str) -> SaveSlot {
    let folder = std::env::temp_dir().join(format!("push-kitchen-test-{test}"));
    // Whatever an earlier run of the same test left behind. Fine if there is nothing.
    let _ = std::fs::remove_dir_all(&folder);
    SaveSlot::File(folder.join("progress.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_solved_at_first() {
        assert!(!Progress::default().is_solved("soup"));
    }

    #[test]
    fn a_marked_level_is_solved() {
        let mut progress = Progress::default();
        progress.mark_solved("soup");
        assert!(progress.is_solved("soup"));
        assert!(!progress.is_solved("salad"));
    }

    #[test]
    fn marking_a_level_is_only_news_the_first_time() {
        let mut progress = Progress::default();
        assert!(progress.mark_solved("soup"));
        assert!(!progress.mark_solved("soup"));
    }

    #[test]
    fn saved_progress_can_be_loaded_again() {
        let slot = scratch_slot("save-and-load");
        let mut progress = Progress::default();
        progress.mark_solved("soup");
        progress.mark_solved("salad");

        progress.save(&slot).expect("saving should work");
        assert_eq!(Progress::load(&slot).expect("loading should work"), progress);
    }

    #[test]
    fn loading_before_anything_was_saved_gives_empty_progress() {
        let slot = scratch_slot("missing");
        assert_eq!(Progress::load(&slot).expect("loading should work"), Progress::default());
    }

    #[test]
    fn loading_skips_blank_lines() {
        let slot = scratch_slot("blank-lines");
        slot.write("soup\n\n  salad  \n").expect("writing should work");

        let progress = Progress::load(&slot).expect("loading should work");
        assert!(progress.is_solved("soup"));
        assert!(progress.is_solved("salad"));
        assert!(!progress.is_solved(""));
    }

    #[test]
    fn on_a_computer_progress_is_saved_to_progress_txt() {
        let SaveSlot::File(file) = SaveSlot::for_this_platform();
        assert_eq!(file.file_name().unwrap(), "progress.txt");
    }
}
