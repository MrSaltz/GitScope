use crate::model::RepositoryStats;

pub fn render(stats: &RepositoryStats) -> serde_json::Result<String> {
    let mut text = serde_json::to_string_pretty(stats)?;
    text.push('\n');
    Ok(text)
}
