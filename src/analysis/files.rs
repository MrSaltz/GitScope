use std::collections::HashMap;
use std::path::Path;

use crate::model::{CommitRecord, ExtensionCount, FileModification, FileStats};

pub(crate) fn extension_of(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .filter(|ext| !ext.is_empty())
}

#[derive(Debug, Default)]
pub struct FilesAccumulator {
    modifications: HashMap<String, usize>,
}

impl FilesAccumulator {
    pub fn add(&mut self, commit: &CommitRecord) {
        for change in &commit.changes.files {
            match self.modifications.get_mut(&change.path) {
                Some(count) => *count += 1,
                None => {
                    self.modifications.insert(change.path.clone(), 1);
                }
            }
        }
    }

    pub fn finish(self, snapshot: &[String]) -> FileStats {
        let mut per_extension: HashMap<Option<String>, usize> = HashMap::new();
        for path in snapshot {
            *per_extension.entry(extension_of(path)).or_default() += 1;
        }
        let mut extensions: Vec<ExtensionCount> = per_extension
            .into_iter()
            .map(|(extension, count)| ExtensionCount { extension, count })
            .collect();
        extensions.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then_with(|| a.extension.cmp(&b.extension))
        });

        let mut most_modified: Vec<FileModification> = self
            .modifications
            .into_iter()
            .map(|(path, modifications)| FileModification {
                path,
                modifications,
            })
            .collect();
        most_modified.sort_by(|a, b| {
            b.modifications
                .cmp(&a.modifications)
                .then_with(|| a.path.cmp(&b.path))
        });

        FileStats {
            total: snapshot.len(),
            extensions,
            most_modified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::testutil::record_with;

    fn snapshot(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|p| (*p).to_owned()).collect()
    }

    #[test]
    fn extension_rules() {
        assert_eq!(extension_of("src/main.rs").as_deref(), Some("rs"));
        assert_eq!(extension_of("a/b/README.MD").as_deref(), Some("md"));
        assert_eq!(extension_of("archive.tar.gz").as_deref(), Some("gz"));
        assert_eq!(extension_of("Makefile"), None);
        assert_eq!(extension_of(".gitignore"), None);
        assert_eq!(extension_of("trailing."), None);
        assert_eq!(extension_of("dir.d/noext"), None);
    }

    #[test]
    fn counts_extensions_and_files_in_the_snapshot() {
        let stats = FilesAccumulator::default().finish(&snapshot(&[
            "src/a.rs",
            "src/b.rs",
            "web/app.ts",
            "README.md",
            "LICENSE",
            "Makefile",
        ]));
        assert_eq!(stats.total, 6);
        let pairs: Vec<_> = stats
            .extensions
            .iter()
            .map(|e| (e.extension.as_deref(), e.count))
            .collect();
        assert_eq!(
            pairs,
            vec![(None, 2), (Some("rs"), 2), (Some("md"), 1), (Some("ts"), 1)]
        );
    }

    #[test]
    fn ranks_most_modified_files_by_commit_count() {
        let mut acc = FilesAccumulator::default();
        acc.add(&record_with(
            "A",
            "a@x",
            "2026-01-01 10:00",
            &["a.rs", "b.rs"],
            1,
            0,
        ));
        acc.add(&record_with(
            "A",
            "a@x",
            "2026-01-02 10:00",
            &["a.rs"],
            1,
            0,
        ));
        acc.add(&record_with(
            "A",
            "a@x",
            "2026-01-03 10:00",
            &["a.rs", "c.rs"],
            1,
            0,
        ));
        let stats = acc.finish(&[]);
        let ranking: Vec<_> = stats
            .most_modified
            .iter()
            .map(|f| (f.path.as_str(), f.modifications))
            .collect();
        assert_eq!(ranking, vec![("a.rs", 3), ("b.rs", 1), ("c.rs", 1)]);
    }
}
