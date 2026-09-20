use std::fmt::Display;

use crate::config::Language;

pub struct Texts {
    pub tabs: [&'static str; 6],
    pub weekdays: [&'static str; 7],

    pub filter_author: &'static str,
    pub filter_branch: &'static str,
    pub filter_since: &'static str,
    pub filter_until: &'static str,
    pub no_commits_yet: &'static str,
    pub no_commits_match: &'static str,

    pub repository: &'static str,
    pub languages: &'static str,
    pub top_contributors: &'static str,
    pub commits: &'static str,
    pub contributors: &'static str,
    pub branches: &'static str,
    pub tags: &'static str,
    pub files: &'static str,
    pub insertions: &'static str,
    pub deletions: &'static str,
    pub period: &'static str,
    pub most_active_hour: &'static str,
    pub no_language: &'static str,
    pub n_commits: &'static str,

    pub contributors_title: &'static str,
    pub col_name: &'static str,
    pub col_email: &'static str,
    pub col_commits: &'static str,
    pub col_files: &'static str,
    pub col_plus_lines: &'static str,
    pub col_minus_lines: &'static str,
    pub col_first: &'static str,
    pub col_last: &'static str,

    pub most_modified: &'static str,
    pub col_path: &'static str,
    pub col_changes: &'static str,
    pub extensions: &'static str,
    pub total_files: &'static str,
    pub no_extension: &'static str,

    pub per_month: &'static str,
    pub per_month_last: &'static str,
    pub by_weekday: &'static str,
    pub by_hour: &'static str,

    pub commits_title: &'static str,
    pub commits_filtered: &'static str,
    pub col_date: &'static str,
    pub col_hash: &'static str,
    pub col_author: &'static str,
    pub col_message: &'static str,
    pub details: &'static str,
    pub no_commit_selected: &'static str,
    pub label_hash: &'static str,
    pub label_author: &'static str,
    pub label_date: &'static str,
    pub label_message: &'static str,
    pub label_files_changed: &'static str,
    pub and_more_files: &'static str,

    pub hints: &'static str,
    pub hints_config: &'static str,
    pub hint_clear_filter: &'static str,
    pub editing_hint: &'static str,

    pub help_title: &'static str,
    pub help_keys: [(&'static str, &'static str); 11],
    pub help_close: &'static str,

    pub config_title: &'static str,
    pub col_setting: &'static str,
    pub col_value: &'static str,
    pub setting_language: &'static str,
    pub config_hint: &'static str,
    pub saved_to: &'static str,
    pub not_stored: &'static str,
    pub save_failed: &'static str,
}

pub fn texts(language: Language) -> &'static Texts {
    match language {
        Language::English => &ENGLISH,
        Language::Portuguese => &PORTUGUESE,
    }
}

pub fn fill(template: &str, values: &[(&str, &dyn Display)]) -> String {
    let mut text = template.to_owned();
    for (name, value) in values {
        text = text.replace(&format!("{{{name}}}"), &value.to_string());
    }
    text
}

static ENGLISH: Texts = Texts {
    tabs: [
        "Overview",
        "Contributors",
        "Files",
        "Activity",
        "Commits",
        "Configurations",
    ],
    weekdays: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],

    filter_author: "author",
    filter_branch: "branch",
    filter_since: "since",
    filter_until: "until",
    no_commits_yet: "This repository has no commits yet.",
    no_commits_match: "No commits match the given filters.",

    repository: "Repository",
    languages: "Languages",
    top_contributors: "Top contributors",
    commits: "Commits",
    contributors: "Contributors",
    branches: "Branches",
    tags: "Tags",
    files: "Files",
    insertions: "Insertions",
    deletions: "Deletions",
    period: "Period",
    most_active_hour: "Most active hour",
    no_language: "No recognised language",
    n_commits: "{n} commits",

    contributors_title: "Contributors ({n}) · Enter: see their commits",
    col_name: "Name",
    col_email: "Email",
    col_commits: "Commits",
    col_files: "Files",
    col_plus_lines: "+Lines",
    col_minus_lines: "-Lines",
    col_first: "First",
    col_last: "Last",

    most_modified: "Most modified files",
    col_path: "Path",
    col_changes: "Changes",
    extensions: "Extensions",
    total_files: "Total files",
    no_extension: "(none)",

    per_month: "Commits per month",
    per_month_last: "Commits per month (last {shown} of {total})",
    by_weekday: "By weekday",
    by_hour: "By hour (UTC)",

    commits_title: "Commits ({n})",
    commits_filtered: "Commits ({shown} of {total}) · filter: {filter}",
    col_date: "Date (UTC)",
    col_hash: "Hash",
    col_author: "Author",
    col_message: "Message",
    details: "Details",
    no_commit_selected: "No commit selected",
    label_hash: "Hash",
    label_author: "Author",
    label_date: "Date",
    label_message: "Message",
    label_files_changed: "Files changed",
    and_more_files: "… and {n} more files",

    hints: "←/→ tabs · ↑/↓ move · Enter open · / filter · ? help · q quit",
    hints_config: "←/→ tabs · ↑/↓ move · Enter/Space change · ? help · q quit",
    hint_clear_filter: "Esc: clear filter",
    editing_hint: "Enter: apply · Esc: clear",

    help_title: "Keys",
    help_keys: [
        ("← → / Tab / h l", "switch tab"),
        ("1 … 6", "go to a tab"),
        ("↑ ↓ / k j", "move the selection"),
        ("PgUp PgDn", "move by 10"),
        ("Home End / g G", "first / last"),
        ("Enter", "on Contributors: see their commits"),
        ("Enter / Space", "on Configurations: change the value"),
        ("/", "filter commits (author, email, message, hash)"),
        ("Esc", "clear the filter"),
        ("?", "this help"),
        ("q / Ctrl+C", "quit"),
    ],
    help_close: "Press any key to close",

    config_title: "Configurations",
    col_setting: "Setting",
    col_value: "Value",
    setting_language: "Language",
    config_hint: "Enter or Space: change the selected setting",
    saved_to: "Saved to {path}",
    not_stored: "No place to store the configuration was found: the choice lasts only until you quit.",
    save_failed: "Could not save: {error}",
};

static PORTUGUESE: Texts = Texts {
    tabs: [
        "Visão geral",
        "Contribuidores",
        "Arquivos",
        "Atividade",
        "Commits",
        "Configurações",
    ],
    weekdays: ["Seg", "Ter", "Qua", "Qui", "Sex", "Sáb", "Dom"],

    filter_author: "autor",
    filter_branch: "branch",
    filter_since: "desde",
    filter_until: "até",
    no_commits_yet: "Este repositório ainda não tem commits.",
    no_commits_match: "Nenhum commit casa com os filtros dados.",

    repository: "Repositório",
    languages: "Linguagens",
    top_contributors: "Principais contribuidores",
    commits: "Commits",
    contributors: "Contribuidores",
    branches: "Branches",
    tags: "Tags",
    files: "Arquivos",
    insertions: "Inserções",
    deletions: "Deleções",
    period: "Período",
    most_active_hour: "Hora mais ativa",
    no_language: "Nenhuma linguagem reconhecida",
    n_commits: "{n} commits",

    contributors_title: "Contribuidores ({n}) · Enter: ver os commits",
    col_name: "Nome",
    col_email: "E-mail",
    col_commits: "Commits",
    col_files: "Arquivos",
    col_plus_lines: "+Linhas",
    col_minus_lines: "-Linhas",
    col_first: "Primeiro",
    col_last: "Último",

    most_modified: "Arquivos mais modificados",
    col_path: "Caminho",
    col_changes: "Mudanças",
    extensions: "Extensões",
    total_files: "Total de arquivos",
    no_extension: "(nenhuma)",

    per_month: "Commits por mês",
    per_month_last: "Commits por mês (últimos {shown} de {total})",
    by_weekday: "Por dia da semana",
    by_hour: "Por hora (UTC)",

    commits_title: "Commits ({n})",
    commits_filtered: "Commits ({shown} de {total}) · filtro: {filter}",
    col_date: "Data (UTC)",
    col_hash: "Hash",
    col_author: "Autor",
    col_message: "Mensagem",
    details: "Detalhes",
    no_commit_selected: "Nenhum commit selecionado",
    label_hash: "Hash",
    label_author: "Autor",
    label_date: "Data",
    label_message: "Mensagem",
    label_files_changed: "Arquivos alterados",
    and_more_files: "… e mais {n} arquivos",

    hints: "←/→ abas · ↑/↓ mover · Enter abrir · / filtrar · ? ajuda · q sair",
    hints_config: "←/→ abas · ↑/↓ mover · Enter/Espaço mudar · ? ajuda · q sair",
    hint_clear_filter: "Esc: limpar filtro",
    editing_hint: "Enter: aplicar · Esc: limpar",

    help_title: "Teclas",
    help_keys: [
        ("← → / Tab / h l", "trocar de aba"),
        ("1 … 6", "ir para uma aba"),
        ("↑ ↓ / k j", "mover a seleção"),
        ("PgUp PgDn", "mover de 10 em 10"),
        ("Home End / g G", "primeiro / último"),
        ("Enter", "em Contribuidores: ver os commits"),
        ("Enter / Espaço", "em Configurações: mudar o valor"),
        ("/", "filtrar commits (autor, e-mail, mensagem, hash)"),
        ("Esc", "limpar o filtro"),
        ("?", "esta ajuda"),
        ("q / Ctrl+C", "sair"),
    ],
    help_close: "Pressione qualquer tecla para fechar",

    config_title: "Configurações",
    col_setting: "Configuração",
    col_value: "Valor",
    setting_language: "Idioma",
    config_hint: "Enter ou Espaço: mudar a configuração selecionada",
    saved_to: "Salvo em {path}",
    not_stored: "Não há onde guardar a configuração: a escolha vale só até você sair.",
    save_failed: "Não foi possível salvar: {error}",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_replaces_every_placeholder() {
        assert_eq!(
            fill("{a} de {b}: {a}", &[("a", &1), ("b", &"dois")]),
            "1 de dois: 1"
        );
        assert_eq!(fill("sem modelo", &[("a", &1)]), "sem modelo");
    }

    #[test]
    fn the_placeholders_survive_in_every_language() {
        for language in Language::ALL {
            let t = texts(language);
            for (template, names) in [
                (t.n_commits, &["n"][..]),
                (t.contributors_title, &["n"]),
                (t.per_month_last, &["shown", "total"]),
                (t.commits_title, &["n"]),
                (t.commits_filtered, &["shown", "total", "filter"]),
                (t.and_more_files, &["n"]),
                (t.saved_to, &["path"]),
                (t.save_failed, &["error"]),
            ] {
                for name in names {
                    assert!(
                        template.contains(&format!("{{{name}}}")),
                        "{language:?}: {template:?} lost {{{name}}}"
                    );
                }
            }
        }
    }

    #[test]
    fn nothing_is_left_empty_and_the_lists_have_distinct_entries() {
        for language in Language::ALL {
            let t = texts(language);
            let mut all: Vec<&str> = t.tabs.to_vec();
            all.extend(t.weekdays);
            all.extend(t.help_keys.iter().flat_map(|(key, what)| [*key, *what]));
            all.extend([
                t.no_commits_yet,
                t.no_commits_match,
                t.repository,
                t.languages,
                t.hints,
                t.help_title,
                t.help_close,
                t.config_title,
                t.setting_language,
                t.config_hint,
                t.not_stored,
            ]);
            assert!(all.iter().all(|s| !s.trim().is_empty()), "{language:?}");

            let unique = |items: &[&str]| {
                let set: std::collections::HashSet<_> = items.iter().collect();
                set.len() == items.len()
            };
            assert!(unique(&t.tabs), "{language:?}: tab titles must differ");
            assert!(unique(&t.weekdays), "{language:?}: weekdays must differ");
        }
    }

    #[test]
    fn the_portuguese_texts_are_really_translated() {
        let (en, pt) = (texts(Language::English), texts(Language::Portuguese));
        assert_ne!(en.tabs[0], pt.tabs[0]);
        assert_ne!(en.no_commits_yet, pt.no_commits_yet);
        assert_ne!(en.help_close, pt.help_close);
        assert_ne!(en.weekdays[5], pt.weekdays[5]);
    }
}
