use chrono::{DateTime, NaiveDate, Utc, Weekday};
use serde::Serialize;

use crate::error::{GitScopeError, Result};

/// Versão do schema JSON; só sobe em mudanças incompatíveis.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Filters {
    pub since: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub author: Option<String>,
    pub branch: Option<String>,
}

impl Filters {
    pub fn parse_date(input: &str) -> Result<NaiveDate> {
        NaiveDate::parse_from_str(input, "%Y-%m-%d").map_err(|_| GitScopeError::InvalidDate {
            input: input.to_owned(),
        })
    }

    pub fn validate(&self) -> Result<()> {
        match (self.since, self.until) {
            (Some(since), Some(until)) if since > until => {
                Err(GitScopeError::InvalidRange { since, until })
            }
            _ => Ok(()),
        }
    }

    pub fn matches_date(&self, date: DateTime<Utc>) -> bool {
        let day = date.date_naive();
        self.since.is_none_or(|since| day >= since) && self.until.is_none_or(|until| day <= until)
    }

    pub fn matches_author(&self, name: &str, email: &str) -> bool {
        let Some(needle) = &self.author else {
            return true;
        };
        let needle = needle.to_lowercase();
        name.to_lowercase().contains(&needle) || email.to_lowercase().contains(&needle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileChange {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    pub kind: ChangeKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitChanges {
    pub files: Vec<FileChange>,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRecord {
    pub id: String,
    pub author: String,
    pub email: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub is_merge: bool,
    /// Mudanças em relação ao (primeiro) pai; vazio em merges.
    pub changes: CommitChanges,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RepositoryStats {
    pub schema_version: u32,
    pub repository: RepositoryInfo,
    pub filters: Filters,
    pub commits: CommitStats,
    pub contributors: Vec<ContributorStats>,
    pub files: FileStats,
    pub languages: Vec<LanguageStats>,
    pub activity: ActivityStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_details: Option<Vec<CommitInfo>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepositoryInfo {
    pub name: String,
    pub branches: usize,
    pub tags: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitStats {
    pub total: usize,
    pub first: Option<DateTime<Utc>>,
    pub last: Option<DateTime<Utc>>,
    pub first_message: Option<String>,
    pub last_message: Option<String>,
    pub insertions: usize,
    pub deletions: usize,
    pub most_active_hour: Option<u32>,
    pub by_weekday: Vec<WeekdayCount>,
    pub by_hour: Vec<HourCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WeekdayCount {
    pub weekday: Weekday,
    pub commits: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HourCount {
    pub hour: u32,
    pub commits: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContributorStats {
    pub name: String,
    pub email: String,
    pub commits: usize,
    pub percentage: f64,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub first_commit: DateTime<Utc>,
    pub last_commit: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileStats {
    /// Arquivos na árvore analisada (retrato da ponta do histórico).
    pub total: usize,
    pub extensions: Vec<ExtensionCount>,
    pub most_modified: Vec<FileModification>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtensionCount {
    pub extension: Option<String>,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileModification {
    pub path: String,
    pub modifications: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LanguageStats {
    pub name: String,
    pub files: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActivityStats {
    pub by_month: Vec<MonthActivity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonthActivity {
    pub month: String,
    pub commits: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub changes: Vec<FileChange>,
}

impl From<CommitRecord> for CommitInfo {
    fn from(record: CommitRecord) -> Self {
        Self {
            hash: record.id,
            author: record.author,
            email: record.email,
            date: record.date,
            message: record.message,
            files_changed: record.changes.files.len(),
            insertions: record.changes.insertions,
            deletions: record.changes.deletions,
            changes: record.changes.files,
        }
    }
}
