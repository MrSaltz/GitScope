#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlKind {
    NotUrl,
    Supported,
    Unsupported,
}

const SUPPORTED_SCHEMES: [&str; 4] = ["https", "http", "git", "file"];

pub fn classify(text: &str) -> UrlKind {
    if let Some((scheme, rest)) = text.split_once("://") {
        let is_scheme = !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
        if is_scheme && !rest.is_empty() {
            let scheme = scheme.to_ascii_lowercase();
            return if SUPPORTED_SCHEMES.contains(&scheme.as_str()) {
                UrlKind::Supported
            } else {
                UrlKind::Unsupported
            };
        }
        return UrlKind::NotUrl;
    }
    if is_scp_like(text) {
        return UrlKind::Unsupported;
    }
    UrlKind::NotUrl
}

fn is_scp_like(text: &str) -> bool {
    let Some((user_host, path)) = text.split_once(':') else {
        return false;
    };
    let Some((user, host)) = user_host.split_once('@') else {
        return false;
    };
    !user.is_empty() && !host.is_empty() && !path.is_empty() && !user_host.contains(['/', '\\'])
}

/// `url` com as credenciais (`usuario:senha@` ou `token@`) trocadas por `***@`,
/// seguro para imprimir ou registrar em log. Sem credenciais, volta igual.
pub fn redact(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_owned();
    };
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(authority_end);
    match authority.rsplit_once('@') {
        Some((_credentials, host)) => format!("{scheme}://***@{host}{tail}"),
        None => url.to_owned(),
    }
}

pub fn repository_name(url: &str) -> String {
    let without_extras = url.split(['?', '#']).next().unwrap_or(url);
    let last = without_extras
        .trim_end_matches('/')
        .rsplit(['/', ':'])
        .next()
        .unwrap_or("");
    let last = last.strip_suffix(".git").unwrap_or(last);
    let cleaned: String = last
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() || cleaned.chars().all(|c| c == '.') {
        "repository".to_owned()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_transports() {
        for url in [
            "https://github.com/owner/repo.git",
            "http://example.com/repo",
            "git://example.com/repo.git",
            "file:///tmp/repo",
            "HTTPS://GitHub.com/owner/repo",
        ] {
            assert_eq!(classify(url), UrlKind::Supported, "{url}");
        }
    }

    #[test]
    fn ssh_and_unknown_transports_are_unsupported() {
        for url in [
            "ssh://git@github.com/owner/repo.git",
            "git+ssh://host/repo",
            "git@github.com:owner/repo.git",
            "user@host:repo",
            "ftp://example.com/repo",
        ] {
            assert_eq!(classify(url), UrlKind::Unsupported, "{url}");
        }
    }

    #[test]
    fn local_paths_are_not_urls() {
        for path in [
            ".",
            "./repo",
            "../repo",
            "/home/me/repo",
            "repo",
            r"C:\Users\me\repo",
            "C:/Users/me/repo",
            r"\\server\share\repo",
            "dir/with:colon",
            "",
            "://missing-scheme",
            "https://",
        ] {
            assert_eq!(classify(path), UrlKind::NotUrl, "{path}");
        }
    }

    #[test]
    fn names_are_derived_from_the_last_path_segment() {
        assert_eq!(repository_name("https://github.com/owner/repo.git"), "repo");
        assert_eq!(repository_name("https://github.com/owner/repo"), "repo");
        assert_eq!(repository_name("https://github.com/owner/repo/"), "repo");
        assert_eq!(repository_name("https://host/repo.git?x=1#frag"), "repo");
        assert_eq!(repository_name("git@github.com:owner/repo.git"), "repo");
        assert_eq!(
            repository_name("file:///C:/Users/me/my project"),
            "my_project"
        );
    }

    #[test]
    fn credentials_are_redacted() {
        assert_eq!(
            redact("https://user:s3cret@github.com/owner/repo.git"),
            "https://***@github.com/owner/repo.git"
        );
        assert_eq!(
            redact("https://ghp_token@github.com/owner/repo.git"),
            "https://***@github.com/owner/repo.git"
        );
        assert_eq!(
            redact("https://u:p@host:8443/a@b?x=y@z"),
            "https://***@host:8443/a@b?x=y@z"
        );
    }

    #[test]
    fn text_without_credentials_is_untouched() {
        for text in [
            "https://github.com/owner/repo.git",
            "https://github.com/owner/a@b.git",
            "file:///C:/Users/me/repo",
            "git@github.com:owner/repo.git",
            "./relative/path",
            "",
        ] {
            assert_eq!(redact(text), text);
        }
    }

    #[test]
    fn names_are_always_safe_directory_names() {
        assert_eq!(repository_name("https://host/"), "host");
        assert_eq!(repository_name("https://host/.."), "repository");
        assert_eq!(repository_name("https://host/a%20b"), "a_20b");
        assert_eq!(repository_name(""), "repository");
    }
}
