use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RenameEntry {
    pub original: PathBuf,
    pub new_name: String,
    pub selected: bool,
}

impl RenameEntry {
    pub fn new(original: PathBuf) -> Self {
        let new_name = original
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        
        Self {
            original,
            new_name,
            selected: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenameMode {
    Simple,
    FindReplace,
    Regex,
    Sequential,
    Lowercase,
    Uppercase,
    Capitalize,
}

#[derive(Debug, Clone)]
pub struct BatchRenamer {
    files: Vec<RenameEntry>,
    mode: RenameMode,
    find_text: String,
    replace_text: String,
    regex_pattern: String,
    regex_replacement: String,
    sequential_start: usize,
    sequential_padding: usize,
    sequential_prefix: String,
    undo_stack: Vec<Vec<(PathBuf, PathBuf)>>,
}

impl BatchRenamer {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            mode: RenameMode::Simple,
            find_text: String::new(),
            replace_text: String::new(),
            regex_pattern: String::new(),
            regex_replacement: String::new(),
            sequential_start: 1,
            sequential_padding: 3,
            sequential_prefix: String::new(),
            undo_stack: Vec::new(),
        }
    }

    pub fn set_files(&mut self, files: Vec<PathBuf>) {
        self.files = files.into_iter().map(RenameEntry::new).collect();
        self.preview();
    }

    pub fn files(&self) -> &[RenameEntry] {
        &self.files
    }

    pub fn files_mut(&mut self) -> &mut Vec<RenameEntry> {
        &mut self.files
    }

    pub fn mode(&self) -> &RenameMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: RenameMode) {
        self.mode = mode;
        self.preview();
    }

    pub fn find_text(&self) -> &str {
        &self.find_text
    }

    pub fn set_find_text(&mut self, text: &str) {
        self.find_text = text.to_string();
        self.preview();
    }

    pub fn replace_text(&self) -> &str {
        &self.replace_text
    }

    pub fn set_replace_text(&mut self, text: &str) {
        self.replace_text = text.to_string();
        self.preview();
    }

    pub fn regex_pattern(&self) -> &str {
        &self.regex_pattern
    }

    pub fn set_regex_pattern(&mut self, pattern: &str) {
        self.regex_pattern = pattern.to_string();
        self.preview();
    }

    pub fn regex_replacement(&self) -> &str {
        &self.regex_replacement
    }

    pub fn set_regex_replacement(&mut self, replacement: &str) {
        self.regex_replacement = replacement.to_string();
        self.preview();
    }

    pub fn sequential_start(&self) -> usize {
        self.sequential_start
    }

    pub fn set_sequential_start(&mut self, start: usize) {
        self.sequential_start = start;
        self.preview();
    }

    pub fn sequential_padding(&self) -> usize {
        self.sequential_padding
    }

    pub fn set_sequential_padding(&mut self, padding: usize) {
        self.sequential_padding = padding;
        self.preview();
    }

    pub fn sequential_prefix(&self) -> &str {
        &self.sequential_prefix
    }

    pub fn set_sequential_prefix(&mut self, prefix: &str) {
        self.sequential_prefix = prefix.to_string();
        self.preview();
    }

    pub fn preview(&mut self) {
        match self.mode {
            RenameMode::FindReplace => self.preview_find_replace(),
            RenameMode::Regex => self.preview_regex(),
            RenameMode::Sequential => self.preview_sequential(),
            RenameMode::Lowercase => self.preview_case(false),
            RenameMode::Uppercase => self.preview_case(true),
            RenameMode::Capitalize => self.preview_capitalize(),
            RenameMode::Simple => {}
        }
    }

    fn preview_find_replace(&mut self) {
        for entry in &mut self.files {
            if !entry.selected {
                continue;
            }

            let old_name = entry.original.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let new_name = old_name.replace(&self.find_text, &self.replace_text);
            entry.new_name = new_name;
        }
    }

    fn preview_regex(&mut self) {
        // Simple regex-like replacement (basic string replacement for now)
        for entry in &mut self.files {
            if !entry.selected {
                continue;
            }

            let old_name = entry.original.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Basic pattern matching (simplified)
            let new_name = if old_name.contains(&self.regex_pattern) {
                old_name.replace(&self.regex_pattern, &self.regex_replacement)
            } else {
                old_name
            };
            entry.new_name = new_name;
        }
    }

    fn preview_sequential(&mut self) {
        let mut counter = self.sequential_start;
        
        for entry in &mut self.files {
            if !entry.selected {
                continue;
            }

            let extension = entry.original.extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();

            let padded_num = format!("{:0width$}", counter, width = self.sequential_padding);
            entry.new_name = format!("{}{}{}", self.sequential_prefix, padded_num, extension);
            counter += 1;
        }
    }

    fn preview_case(&mut self, to_upper: bool) {
        for entry in &mut self.files {
            if !entry.selected {
                continue;
            }

            let old_name = entry.original.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            entry.new_name = if to_upper {
                old_name.to_uppercase()
            } else {
                old_name.to_lowercase()
            };
        }
    }

    fn preview_capitalize(&mut self) {
        for entry in &mut self.files {
            if !entry.selected {
                continue;
            }

            let old_name = entry.original.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            entry.new_name = old_name
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i == 0 || old_name.chars().nth(i - 1) == Some(' ') {
                        c.to_uppercase().to_string()
                    } else {
                        c.to_string()
                    }
                })
                .collect();
        }
    }

    pub fn has_changes(&self) -> bool {
        self.files.iter().any(|e| {
            e.selected && e.new_name != e.original.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        })
    }

    pub fn selected_count(&self) -> usize {
        self.files.iter().filter(|e| e.selected).count()
    }

    pub fn rename_all(&mut self) -> Result<Vec<(PathBuf, PathBuf)>, String> {
        let mut operations = Vec::new();

        for entry in &mut self.files {
            if !entry.selected {
                continue;
            }

            let old_path = entry.original.clone();
            let parent = old_path.parent().ok_or("Cannot determine parent directory")?;
            let new_path = parent.join(&entry.new_name);

            if old_path == new_path {
                continue;
            }

            if new_path.exists() {
                return Err(format!("File already exists: {}", new_path.display()));
            }

            std::fs::rename(&old_path, &new_path)
                .map_err(|e| format!("Failed to rename {}: {}", old_path.display(), e))?;

            operations.push((old_path, new_path.clone()));
            entry.original = new_path;
        }

        if !operations.is_empty() {
            self.undo_stack.push(operations.clone());
        }

        Ok(operations)
    }

    pub fn undo(&mut self) -> Result<Vec<(PathBuf, PathBuf)>, String> {
        let operations = self.undo_stack.pop().ok_or("Nothing to undo")?;

        let mut undone = Vec::new();
        for (old_path, new_path) in operations.into_iter().rev() {
            if new_path.exists() {
                std::fs::rename(&new_path, &old_path)
                    .map_err(|e| format!("Failed to undo: {}", e))?;
                undone.push((new_path, old_path));
            }
        }

        Ok(undone)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
}

impl Default for BatchRenamer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_renamer_new() {
        let renamer = BatchRenamer::new();
        assert!(renamer.files().is_empty());
        assert_eq!(*renamer.mode(), RenameMode::Simple);
    }

    #[test]
    fn test_batch_renamer_set_files() {
        let mut renamer = BatchRenamer::new();
        let files = vec![
            PathBuf::from("file1.txt"),
            PathBuf::from("file2.txt"),
        ];
        renamer.set_files(files);
        assert_eq!(renamer.files().len(), 2);
    }

    #[test]
    fn test_batch_renamer_find_replace() {
        let mut renamer = BatchRenamer::new();
        renamer.set_files(vec![
            PathBuf::from("photo_vacation.jpg"),
            PathBuf::from("photo_summer.jpg"),
        ]);
        renamer.set_mode(RenameMode::FindReplace);
        renamer.set_find_text("photo_");
        renamer.set_replace_text("image_");

        assert_eq!(renamer.files()[0].new_name, "image_vacation.jpg");
        assert_eq!(renamer.files()[1].new_name, "image_summer.jpg");
    }

    #[test]
    fn test_batch_renamer_sequential() {
        let mut renamer = BatchRenamer::new();
        renamer.set_files(vec![
            PathBuf::from("old1.txt"),
            PathBuf::from("old2.txt"),
            PathBuf::from("old3.txt"),
        ]);
        renamer.set_mode(RenameMode::Sequential);
        renamer.set_sequential_prefix("doc_");
        renamer.set_sequential_start(1);
        renamer.set_sequential_padding(2);

        assert_eq!(renamer.files()[0].new_name, "doc_01.txt");
        assert_eq!(renamer.files()[1].new_name, "doc_02.txt");
        assert_eq!(renamer.files()[2].new_name, "doc_03.txt");
    }

    #[test]
    fn test_batch_renamer_lowercase() {
        let mut renamer = BatchRenamer::new();
        renamer.set_files(vec![PathBuf::from("FILE.TXT")]);
        renamer.set_mode(RenameMode::Lowercase);

        assert_eq!(renamer.files()[0].new_name, "file.txt");
    }

    #[test]
    fn test_batch_renamer_uppercase() {
        let mut renamer = BatchRenamer::new();
        renamer.set_files(vec![PathBuf::from("file.txt")]);
        renamer.set_mode(RenameMode::Uppercase);

        assert_eq!(renamer.files()[0].new_name, "FILE.TXT");
    }

    #[test]
    fn test_batch_renamer_has_changes() {
        let mut renamer = BatchRenamer::new();
        renamer.set_files(vec![PathBuf::from("file.txt")]);
        assert!(!renamer.has_changes());

        renamer.set_mode(RenameMode::FindReplace);
        renamer.set_find_text("file");
        renamer.set_replace_text("document");
        assert!(renamer.has_changes());
    }

    #[test]
    fn test_batch_renamer_selected_count() {
        let mut renamer = BatchRenamer::new();
        renamer.set_files(vec![
            PathBuf::from("file1.txt"),
            PathBuf::from("file2.txt"),
            PathBuf::from("file3.txt"),
        ]);
        assert_eq!(renamer.selected_count(), 3);

        renamer.files_mut()[1].selected = false;
        assert_eq!(renamer.selected_count(), 2);
    }

    #[test]
    fn test_batch_renamer_undo() {
        let renamer = BatchRenamer::new();
        assert!(!renamer.can_undo());
    }
}
