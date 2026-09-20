use std::path::PathBuf;

use chrono::NaiveDate;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GitScopeError {
    #[error("path does not exist: {}", .0.display())]
    PathNotFound(PathBuf),

    #[error("not a Git repository: {}", .0.display())]
    NotARepository(PathBuf),

    #[error("branch not found: '{0}'")]
    BranchNotFound(String),

    #[error("invalid date '{input}': expected the format YYYY-MM-DD")]
    InvalidDate { input: String },

    #[error("invalid date range: --since ({since}) is after --until ({until})")]
    InvalidRange { since: NaiveDate, until: NaiveDate },

    #[error("unsupported repository URL '{}': {reason}", crate::remote::redact(.url))]
    UnsupportedUrl { url: String, reason: &'static str },

    #[error("failed to clone '{}': {}{}", crate::remote::redact(.url), clone_reason(.cause), auth_hint(.cause))]
    CloneFailed { url: String, cause: git2::Error },

    #[error(transparent)]
    Markdown(#[from] MarkdownError),

    #[error("cannot {action} {}: {cause}", path.display())]
    File {
        action: &'static str,
        path: PathBuf,
        cause: std::io::Error,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum MarkdownError {
    #[error("line {line}: found <!-- gitscope:end --> without a matching <!-- gitscope:start -->")]
    UnmatchedEnd { line: usize },

    #[error(
        "line {line}: <!-- gitscope:start --> is never closed (add <!-- gitscope:end --> after it)"
    )]
    UnclosedBlock { line: usize },

    #[error(
        "line {line}: <!-- gitscope:start --> found inside the block opened at line {open_line}; blocks cannot be nested, and each one needs its own <!-- gitscope:end -->"
    )]
    NestedBlock { line: usize, open_line: usize },

    #[error(
        "line {line}: the stored template comment (<!-- gitscope:template) is never closed by a line containing only -->"
    )]
    UnterminatedTemplate { line: usize },

    #[error(
        "line {line}: the block contains \"-->\", which cannot be stored inside the HTML comment that keeps its template. Remove it, or render with `gitscope markdown --output` from a separate template file"
    )]
    UnstorableTemplate { line: usize },

    #[error(
        "no GitScope block found: wrap the part to update in <!-- gitscope:start --> and <!-- gitscope:end -->"
    )]
    NoBlocks,

    #[error("{}", describe_variable_problems(.problems, .available))]
    Variables {
        problems: Vec<VariableProblem>,
        available: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableProblem {
    pub line: usize,
    pub token: String,
    pub kind: ProblemKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProblemKind {
    Unknown { suggestion: Option<&'static str> },
    Invalid(String),
}

fn describe_variable_problems(problems: &[VariableProblem], available: &[String]) -> String {
    let mut text = String::new();
    for problem in problems {
        match &problem.kind {
            ProblemKind::Unknown { suggestion } => {
                text.push_str(&format!(
                    "Unknown GitScope variable: {} (line {})
",
                    problem.token, problem.line
                ));
                if let Some(name) = suggestion {
                    text.push_str(&format!(
                        "  Did you mean {{{name}}}?
"
                    ));
                }
            }
            ProblemKind::Invalid(reason) => text.push_str(&format!(
                "Invalid GitScope variable: {} (line {}): {reason}
",
                problem.token, problem.line
            )),
        }
    }
    text.push_str(&format!(
        "Available variables: {}
",
        available.join(", ")
    ));
    text.push_str(
        "Run `gitscope variables` for descriptions, or use --keep-unknown to leave unknown variables untouched.",
    );
    text
}

pub type Result<T> = std::result::Result<T, GitScopeError>;

/// As mensagens da libgit2 trazem um sufixo `; class=...` (às vezes repetido);
/// só a parte legível interessa.
fn clone_reason(err: &git2::Error) -> &str {
    err.message().split("; class=").next().unwrap_or_default()
}

/// Hosts respondem 401/403 tanto para repositórios privados quanto para
/// inexistentes.
fn auth_hint(err: &git2::Error) -> &'static str {
    let message = err.message().to_lowercase();
    if !cfg!(feature = "remote")
        && (message.contains("unsupported url protocol") || message.contains("no tls stream"))
    {
        " (this build has no HTTP(S) support; reinstall with the default features)"
    } else if message.contains("401")
        || message.contains("403")
        || message.contains("authentication")
    {
        " (the repository may not exist or may be private; GitScope does not support authentication)"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clone_failed(message: &str) -> String {
        GitScopeError::CloneFailed {
            url: "https://example.com/r.git".into(),
            cause: git2::Error::from_str(message),
        }
        .to_string()
    }

    #[test]
    fn clone_errors_drop_the_libgit2_class_suffix() {
        let text =
            clone_failed("connection refused; class=Os (2): connection refused; class=Os (2)");
        assert_eq!(
            text,
            "failed to clone 'https://example.com/r.git': connection refused"
        );
    }

    #[test]
    fn credentials_in_urls_never_reach_error_messages() {
        let text = GitScopeError::CloneFailed {
            url: "https://user:s3cret@example.com/r.git".into(),
            cause: git2::Error::from_str("boom"),
        }
        .to_string();
        assert!(!text.contains("s3cret") && !text.contains("user"), "{text}");
        assert!(text.contains("https://***@example.com/r.git"));

        let text = GitScopeError::UnsupportedUrl {
            url: "ssh://user:s3cret@example.com/r.git".into(),
            reason: "no",
        }
        .to_string();
        assert!(!text.contains("s3cret"), "{text}");
    }

    #[test]
    fn authentication_failures_explain_the_limitation() {
        let text = clone_failed("request failed with status code: 401");
        assert!(text.contains("status code: 401"));
        assert!(text.contains("may not exist or may be private"));
        assert!(text.contains("does not support authentication"));
    }
}
