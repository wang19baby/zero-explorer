use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: [u8; 3],
}

impl Tag {
    pub fn new(id: &str, name: &str, color: [u8; 3]) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            color,
        }
    }

    pub fn red() -> Self {
        Self::new("red", "Red", [255, 0, 0])
    }

    pub fn orange() -> Self {
        Self::new("orange", "Orange", [255, 153, 0])
    }

    pub fn yellow() -> Self {
        Self::new("yellow", "Yellow", [255, 255, 0])
    }

    pub fn green() -> Self {
        Self::new("green", "Green", [0, 255, 0])
    }

    pub fn blue() -> Self {
        Self::new("blue", "Blue", [0, 0, 255])
    }

    pub fn purple() -> Self {
        Self::new("purple", "Purple", [128, 0, 128])
    }

    pub fn pink() -> Self {
        Self::new("pink", "Pink", [255, 102, 178])
    }

    pub fn gray() -> Self {
        Self::new("gray", "Gray", [128, 128, 128])
    }
}

#[derive(Debug, Clone)]
pub struct TagManager {
    tags: HashMap<String, Tag>,
    file_tags: HashMap<PathBuf, Vec<String>>,
}

impl TagManager {
    pub fn new() -> Self {
        let mut manager = Self {
            tags: HashMap::new(),
            file_tags: HashMap::new(),
        };
        manager.register_default_tags();
        manager
    }

    fn register_default_tags(&mut self) {
        self.register_tag(Tag::red());
        self.register_tag(Tag::orange());
        self.register_tag(Tag::yellow());
        self.register_tag(Tag::green());
        self.register_tag(Tag::blue());
        self.register_tag(Tag::purple());
        self.register_tag(Tag::pink());
        self.register_tag(Tag::gray());
    }

    pub fn register_tag(&mut self, tag: Tag) {
        self.tags.insert(tag.id.clone(), tag);
    }

    pub fn unregister_tag(&mut self, id: &str) -> Option<Tag> {
        self.tags.remove(id)
    }

    pub fn get_tag(&self, id: &str) -> Option<&Tag> {
        self.tags.get(id)
    }

    pub fn all_tags(&self) -> Vec<&Tag> {
        self.tags.values().collect()
    }

    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    pub fn add_tag_to_file(&mut self, file_path: &Path, tag_id: &str) -> bool {
        if !self.tags.contains_key(tag_id) {
            return false;
        }

        let tags = self.file_tags.entry(file_path.to_path_buf()).or_default();
        if !tags.contains(&tag_id.to_string()) {
            tags.push(tag_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn remove_tag_from_file(&mut self, file_path: &Path, tag_id: &str) -> bool {
        if let Some(tags) = self.file_tags.get_mut(file_path) {
            if let Some(pos) = tags.iter().position(|t| t == tag_id) {
                tags.remove(pos);
                if tags.is_empty() {
                    self.file_tags.remove(file_path);
                }
                return true;
            }
        }
        false
    }

    pub fn file_tags(&self, file_path: &Path) -> Vec<&Tag> {
        self.file_tags
            .get(file_path)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.tags.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn files_with_tag(&self, tag_id: &str) -> Vec<&PathBuf> {
        self.file_tags
            .iter()
            .filter(|(_, tags)| tags.contains(&tag_id.to_string()))
            .map(|(path, _)| path)
            .collect()
    }

    pub fn files_with_any_tags(&self, tag_ids: &[String]) -> Vec<&PathBuf> {
        self.file_tags
            .iter()
            .filter(|(_, tags)| tags.iter().any(|t| tag_ids.contains(t)))
            .map(|(path, _)| path)
            .collect()
    }

    pub fn clear_file_tags(&mut self, file_path: &PathBuf) {
        self.file_tags.remove(file_path);
    }
}

impl Default for TagManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_new() {
        let tag = Tag::new("custom", "Custom", [128, 128, 128]);
        assert_eq!(tag.id, "custom");
        assert_eq!(tag.name, "Custom");
    }

    #[test]
    fn test_tag_default_colors() {
        let red = Tag::red();
        assert_eq!(red.color, [255, 0, 0]);
        
        let blue = Tag::blue();
        assert_eq!(blue.color, [0, 0, 255]);
    }

    #[test]
    fn test_tag_manager_new() {
        let manager = TagManager::new();
        assert!(manager.tag_count() >= 8); // At least 8 default tags
    }

    #[test]
    fn test_tag_manager_register_tag() {
        let mut manager = TagManager::new();
        let tag = Tag::new("custom", "Custom", [128, 128, 128]);
        
        manager.register_tag(tag);
        assert!(manager.get_tag("custom").is_some());
    }

    #[test]
    fn test_tag_manager_add_tag_to_file() {
        let mut manager = TagManager::new();
        let file = PathBuf::from("test.txt");
        
        assert!(manager.add_tag_to_file(&file, "red"));
        assert!(!manager.add_tag_to_file(&file, "red")); // Already added
        
        let tags = manager.file_tags(&file);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].id, "red");
    }

    #[test]
    fn test_tag_manager_remove_tag_from_file() {
        let mut manager = TagManager::new();
        let file = PathBuf::from("test.txt");
        
        manager.add_tag_to_file(&file, "red");
        assert!(manager.remove_tag_from_file(&file, "red"));
        assert!(!manager.remove_tag_from_file(&file, "red")); // Already removed
        
        let tags = manager.file_tags(&file);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_tag_manager_files_with_tag() {
        let mut manager = TagManager::new();
        let file1 = PathBuf::from("file1.txt");
        let file2 = PathBuf::from("file2.txt");
        
        manager.add_tag_to_file(&file1, "red");
        manager.add_tag_to_file(&file2, "red");
        manager.add_tag_to_file(&file1, "blue");
        
        let files = manager.files_with_tag("red");
        assert_eq!(files.len(), 2);
        
        let files = manager.files_with_tag("blue");
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_tag_manager_files_with_any_tags() {
        let mut manager = TagManager::new();
        let file1 = PathBuf::from("file1.txt");
        let file2 = PathBuf::from("file2.txt");
        let file3 = PathBuf::from("file3.txt");
        
        manager.add_tag_to_file(&file1, "red");
        manager.add_tag_to_file(&file2, "blue");
        manager.add_tag_to_file(&file3, "green");
        
        let files = manager.files_with_any_tags(&["red".to_string(), "blue".to_string()]);
        assert_eq!(files.len(), 2);
    }
}
