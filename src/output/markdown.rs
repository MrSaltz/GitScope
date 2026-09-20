use std::fs;
use std::io::Write;
use std::path::Path;

use super::variables::{Context, ResolveError, VariableRegistry};
use crate::error::{GitScopeError, MarkdownError, ProblemKind, Result, VariableProblem};

const MARKER_NAMES: [&str; 3] = ["gitscope", "gitstatus", "gitstats"];
const TEMPLATE_OPENER: &str = "<!-- gitscope:template";
const TEMPLATE_CLOSER: &str = "-->";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Render,
    Update,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnknownPolicy {
    #[default]
    Error,
    Keep,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub mode: Mode,
    pub unknown: UnknownPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    pub text: String,
    pub blocks: usize,
    pub replacements: usize,
    pub warnings: Vec<VariableProblem>,
}

pub fn render(
    source: &str,
    registry: &VariableRegistry,
    context: &Context<'_>,
    options: &Options,
) -> std::result::Result<Rendered, MarkdownError> {
    let lines = split_lines(source);
    let blocks = find_blocks(source, &lines)?;
    let eol = line_ending(source);
    let mut resolver = Resolver {
        registry,
        context,
        policy: options.unknown,
        problems: Vec::new(),
        warnings: Vec::new(),
        replacements: 0,
    };

    let text = if blocks.is_empty() {
        if options.mode == Mode::Update {
            return Err(MarkdownError::NoBlocks);
        }
        resolver.substitute(source, 1)
    } else {
        let mut out = String::with_capacity(source.len());
        let mut cursor = 0;
        for block in &blocks {
            out.push_str(&source[cursor..block.body.start]);
            out.push_str(&process_block(
                source,
                &lines,
                block,
                options.mode,
                eol,
                &mut resolver,
            )?);
            cursor = block.body.end;
        }
        out.push_str(&source[cursor..]);
        out
    };

    if !resolver.problems.is_empty() {
        return Err(MarkdownError::Variables {
            problems: resolver.problems,
            available: registry.iter().map(|d| d.usage()).collect(),
        });
    }
    Ok(Rendered {
        text,
        blocks: blocks.len(),
        replacements: resolver.replacements,
        warnings: resolver.warnings,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOutcome {
    pub changed: bool,
    pub blocks: usize,
    pub replacements: usize,
    pub warnings: Vec<VariableProblem>,
}

pub fn check_structure(source: &str, mode: Mode) -> std::result::Result<usize, MarkdownError> {
    let lines = split_lines(source);
    let blocks = find_blocks(source, &lines)?;
    if blocks.is_empty() && mode == Mode::Update {
        return Err(MarkdownError::NoBlocks);
    }
    for block in &blocks {
        stored_template(source, &lines, block)?;
    }
    Ok(blocks.len())
}

pub fn read_document(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|cause| GitScopeError::File {
        action: "read",
        path: path.to_path_buf(),
        cause,
    })
}

pub fn update_file(
    path: &Path,
    registry: &VariableRegistry,
    context: &Context<'_>,
    unknown: UnknownPolicy,
) -> Result<UpdateOutcome> {
    let source = read_document(path)?;
    let options = Options {
        mode: Mode::Update,
        unknown,
    };
    let rendered = render(&source, registry, context, &options)?;
    let changed = rendered.text != source;
    if changed {
        write_atomic(path, &rendered.text)?;
    }
    Ok(UpdateOutcome {
        changed,
        blocks: rendered.blocks,
        replacements: rendered.replacements,
        warnings: rendered.warnings,
    })
}

/// Grava de forma atômica: temporário ao lado, `sync` e `rename` sobre o destino,
/// para leitores nunca verem o arquivo pela metade (um symlink é seguido).
pub fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let fail = |cause: std::io::Error| GitScopeError::File {
        action: "write",
        path: path.to_path_buf(),
        cause,
    };

    let is_symlink = fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink());
    let target = if is_symlink {
        fs::canonicalize(path).map_err(fail)?
    } else {
        path.to_path_buf()
    };
    let directory = match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };

    let mut temp = tempfile::Builder::new()
        .prefix(".gitscope-")
        .suffix(".tmp")
        .tempfile_in(directory)
        .map_err(fail)?;
    temp.write_all(contents.as_bytes()).map_err(fail)?;
    temp.flush().map_err(fail)?;
    temp.as_file().sync_all().map_err(fail)?;
    if let Ok(metadata) = fs::metadata(&target) {
        fs::set_permissions(temp.path(), metadata.permissions()).map_err(fail)?;
    }
    temp.persist(&target).map_err(|e| fail(e.error))?;
    Ok(())
}

struct Line<'a> {
    number: usize,
    start: usize,
    next: usize,
    text: &'a str,
}

fn split_lines(source: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, raw) in source.split_inclusive('\n').enumerate() {
        let text = match raw.strip_suffix('\n') {
            Some(without) => without.strip_suffix('\r').unwrap_or(without),
            None => raw,
        };
        // Um BOM faz parte do arquivo, não do texto da primeira linha.
        let text = if index == 0 {
            text.strip_prefix('\u{feff}').unwrap_or(text)
        } else {
            text
        };
        lines.push(Line {
            number: index + 1,
            start,
            next: start + raw.len(),
            text,
        });
        start += raw.len();
    }
    lines
}

fn line_ending(source: &str) -> &'static str {
    match source.find('\n') {
        Some(at) if source[..at].ends_with('\r') => "\r\n",
        _ => "\n",
    }
}

#[derive(Debug)]
struct Block {
    start_line: usize,
    body: std::ops::Range<usize>,
    body_lines: std::ops::Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Marker {
    Start,
    End,
}

fn parse_marker(line: &str) -> Option<Marker> {
    if line.starts_with(' ') && line.len() - line.trim_start_matches(' ').len() > 3 {
        return None;
    }
    let inner = line
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    let (name, kind) = inner.split_once(':')?;
    if !MARKER_NAMES.contains(&name.trim()) {
        return None;
    }
    match kind.trim() {
        "start" => Some(Marker::Start),
        "end" => Some(Marker::End),
        _ => None,
    }
}

/// Acompanha blocos de código com cerca, onde os marcadores são documentação, e
/// não marcadores.
#[derive(Default)]
struct Fence(Option<(char, usize)>);

impl Fence {
    fn is_code(&mut self, line: &str) -> bool {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let trimmed = &line[indent..];
        let ch = trimmed.chars().next();
        let run = match ch {
            Some(c @ ('`' | '~')) => trimmed.chars().take_while(|&x| x == c).count(),
            _ => 0,
        };
        let rest = trimmed.get(run..).unwrap_or("");

        match self.0 {
            Some((open_char, open_len)) => {
                if indent <= 3 && ch == Some(open_char) && run >= open_len && rest.trim().is_empty()
                {
                    self.0 = None;
                }
                true
            }
            None => {
                let opens = indent <= 3 && run >= 3 && (ch == Some('~') || !rest.contains('`'));
                if opens {
                    self.0 = ch.map(|c| (c, run));
                }
                opens
            }
        }
    }
}

fn find_blocks(source: &str, lines: &[Line<'_>]) -> std::result::Result<Vec<Block>, MarkdownError> {
    let mut blocks = Vec::new();
    let mut open: Option<(usize, usize)> = None;
    let mut fence = Fence::default();

    for (index, line) in lines.iter().enumerate() {
        if fence.is_code(line.text) {
            continue;
        }
        match parse_marker(line.text) {
            Some(Marker::Start) => {
                if let Some((open_line, _)) = open {
                    return Err(MarkdownError::NestedBlock {
                        line: line.number,
                        open_line,
                    });
                }
                open = Some((line.number, index + 1));
            }
            Some(Marker::End) => {
                let Some((start_line, first)) = open.take() else {
                    return Err(MarkdownError::UnmatchedEnd { line: line.number });
                };
                let body_start = lines
                    .get(first)
                    .map_or(source.len(), |l| l.start)
                    .min(line.start);
                blocks.push(Block {
                    start_line,
                    body: body_start..line.start,
                    body_lines: first..index,
                });
            }
            None => {}
        }
    }
    match open {
        Some((line, _)) => Err(MarkdownError::UnclosedBlock { line }),
        None => Ok(blocks),
    }
}

fn is_template_opener(line: &str) -> bool {
    let line = line.trim_end();
    MARKER_NAMES
        .iter()
        .any(|name| line == format!("<!-- {name}:template"))
}

struct Stored<'a> {
    text: &'a str,
    first_line: usize,
}

fn stored_template<'a>(
    source: &'a str,
    lines: &[Line<'a>],
    block: &Block,
) -> std::result::Result<Option<Stored<'a>>, MarkdownError> {
    let body = &lines[block.body_lines.clone()];
    let Some((opener_at, opener)) = body
        .iter()
        .enumerate()
        .find(|(_, l)| !l.text.trim().is_empty())
    else {
        return Ok(None);
    };
    if !is_template_opener(opener.text) {
        return Ok(None);
    }
    let closer = body[opener_at + 1..]
        .iter()
        .find(|l| l.text.trim() == TEMPLATE_CLOSER)
        .ok_or(MarkdownError::UnterminatedTemplate {
            line: opener.number,
        })?;
    Ok(Some(Stored {
        text: &source[opener.next..closer.start],
        first_line: opener.number + 1,
    }))
}

fn process_block(
    source: &str,
    lines: &[Line<'_>],
    block: &Block,
    mode: Mode,
    eol: &str,
    resolver: &mut Resolver<'_, '_>,
) -> std::result::Result<String, MarkdownError> {
    let original = &source[block.body.clone()];
    let stored = stored_template(source, lines, block)?;
    let (template, first_line) = match &stored {
        Some(s) => (s.text, s.first_line),
        None => (original, block.start_line + 1),
    };

    let rendered = resolver.substitute(template, first_line);
    Ok(match mode {
        Mode::Render => rendered,
        Mode::Update => {
            if stored.is_none() && (template.trim().is_empty() || rendered == template) {
                original.to_owned()
            } else if template.contains(TEMPLATE_CLOSER) {
                return Err(MarkdownError::UnstorableTemplate {
                    line: block.start_line,
                });
            } else {
                format!("{TEMPLATE_OPENER}{eol}{template}{TEMPLATE_CLOSER}{eol}{rendered}")
            }
        }
    })
}

struct Resolver<'r, 'c> {
    registry: &'r VariableRegistry,
    context: &'r Context<'c>,
    policy: UnknownPolicy,
    problems: Vec<VariableProblem>,
    warnings: Vec<VariableProblem>,
    replacements: usize,
}

struct Token<'a> {
    len: usize,
    name: &'a str,
    args: Vec<&'a str>,
}

fn parse_token(text: &str) -> Option<Token<'_>> {
    let bytes = text.as_bytes();
    let mut end = 1;
    match bytes.get(end) {
        Some(b) if b.is_ascii_alphabetic() || *b == b'_' => end += 1,
        _ => return None,
    }
    while bytes
        .get(end)
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
    {
        end += 1;
    }
    let name = &text[1..end];

    let mut args = Vec::new();
    loop {
        match bytes.get(end) {
            Some(b'}') => {
                return Some(Token {
                    len: end + 1,
                    name,
                    args,
                });
            }
            Some(b':') => {
                let start = end + 1;
                let mut stop = start;
                while bytes
                    .get(stop)
                    .is_some_and(|b| !matches!(b, b'{' | b'}' | b':' | b'\n' | b'\r'))
                {
                    stop += 1;
                }
                if !matches!(bytes.get(stop), Some(b'}' | b':')) {
                    return None;
                }
                args.push(&text[start..stop]);
                end = stop;
            }
            _ => return None,
        }
    }
}

/// Impede que um valor abra ou feche comentários HTML (e quebre o bloco em que
/// entra).
fn neutralize(value: &str) -> String {
    value.replace("<!--", "&lt;!--").replace("-->", "--&gt;")
}

impl Resolver<'_, '_> {
    fn substitute(&mut self, template: &str, first_line: usize) -> String {
        let mut out = String::with_capacity(template.len());
        let mut line = first_line;
        let mut rest = template;
        let mut previous: Option<char> = None;

        while let Some(at) = rest.find(['{', '\\']) {
            let (plain, tail) = rest.split_at(at);
            out.push_str(plain);
            line += plain.matches('\n').count();
            previous = plain.chars().next_back().or(previous);

            if let Some(escaped) = tail.strip_prefix('\\') {
                // `\{nome}` é um `{nome}` literal; qualquer outra barra invertida fica como está.
                let literal = escaped
                    .starts_with('{')
                    .then(|| parse_token(escaped))
                    .flatten();
                if let Some(token) = literal {
                    out.push_str(&escaped[..token.len]);
                    previous = Some('}');
                    rest = &escaped[token.len..];
                } else {
                    out.push('\\');
                    previous = Some('\\');
                    rest = escaped;
                }
                continue;
            }

            let foreign = matches!(previous, Some('$' | '{'));
            match parse_token(tail).filter(|_| !foreign) {
                Some(token) => {
                    let text = &tail[..token.len];
                    out.push_str(&self.resolve(&token, text, line));
                    previous = Some('}');
                    rest = &tail[token.len..];
                }
                None => {
                    out.push('{');
                    previous = Some('{');
                    rest = &tail[1..];
                }
            }
        }
        out.push_str(rest);
        out
    }

    fn resolve(&mut self, token: &Token<'_>, text: &str, line: usize) -> String {
        match self.registry.resolve(self.context, token.name, &token.args) {
            Ok(value) => {
                self.replacements += 1;
                neutralize(&value)
            }
            Err(error) => {
                let kind = match error {
                    ResolveError::Unknown => ProblemKind::Unknown {
                        suggestion: self.registry.suggest(token.name),
                    },
                    ResolveError::Arguments(reason) => ProblemKind::Invalid(reason),
                };
                let problem = VariableProblem {
                    line,
                    token: text.to_owned(),
                    kind,
                };
                match self.policy {
                    UnknownPolicy::Error => self.problems.push(problem),
                    UnknownPolicy::Keep => self.warnings.push(problem),
                }
                text.to_owned()
            }
        }
    }
}

#[cfg(test)]
mod tests;
