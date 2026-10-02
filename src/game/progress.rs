use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bevy::prelude::*;

/// Which kitchens have been solved. Kept in a file between runs of the game.
#[derive(Resource, Debug, Default, PartialEq)]
pub struct Progress {
    /// Level ids. A `BTreeSet` keeps them sorted, so the file comes out the
    /// same whatever order the levels were solved in.
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

    /// Reads a progress file: one level id per line. If there is no file yet,
    /// nothing has been solved yet.
    pub fn load(file: &Path) -> io::Result<Progress> {
        let text = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error),
        };
        let solved = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();
        Ok(Progress { solved })
    }

    /// Writes the progress file, creating its folder if needed.
    pub fn save(&self, file: &Path) -> io::Result<()> {
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder)?;
        }
        let lines: Vec<&str> = self.solved.iter().map(String::as_str).collect();
        fs::write(file, lines.join("\n"))
    }
}

/// Where the progress file is kept.
#[derive(Resource)]
pub struct ProgressFile(pub PathBuf);

/// The progress file of the real game, in the place macOS gives each app for
/// its own data. Without a home folder, it goes next to where the game runs.
pub fn default_progress_file() -> PathBuf {
    let folder = match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home).join("Library/Application Support/Push Kitchen"),
        None => PathBuf::new(),
    };
    folder.join("progress.txt")
}

/// A progress file that belongs to one test only, in the system's temp folder.
/// It does not exist yet when the test starts.
#[cfg(test)]
pub fn scratch_file(test: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("push-kitchen-test-{test}"));
    // Whatever an earlier run of the same test left behind. Fine if there is nothing.
    let _ = fs::remove_dir_all(&folder);
    folder.join("progress.txt")
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
        let file = scratch_file("save-and-load");
        let mut progress = Progress::default();
        progress.mark_solved("soup");
        progress.mark_solved("salad");

        progress.save(&file).expect("saving should work");
        assert_eq!(Progress::load(&file).expect("loading should work"), progress);
    }

    #[test]
    fn loading_a_missing_file_gives_empty_progress() {
        let file = scratch_file("missing");
        assert_eq!(Progress::load(&file).expect("loading should work"), Progress::default());
    }

    #[test]
    fn loading_skips_blank_lines() {
        let file = scratch_file("blank-lines");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "soup\n\n  salad  \n").unwrap();

        let progress = Progress::load(&file).expect("loading should work");
        assert!(progress.is_solved("soup"));
        assert!(progress.is_solved("salad"));
        assert!(!progress.is_solved(""));
    }

    #[test]
    fn the_real_progress_file_is_called_progress_txt() {
        assert_eq!(
            default_progress_file().file_name().unwrap(),
            "progress.txt"
        );
    }
}
