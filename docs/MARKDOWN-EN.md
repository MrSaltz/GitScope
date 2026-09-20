# Markdown templates

[Português](MARKDOWN.md) · **English**

GitScope can fill your README with statistics. You write the Markdown, exactly the way you want it, and put **variables** such as `{commits}` where the numbers should go. GitScope replaces **only the variables**; everything else stays as you wrote it.

```markdown
🚀 This project has {commits} commits from {contributors} contributors.
```

becomes

```markdown
🚀 This project has 361 commits from 2 contributors.
```

GitScope does not impose a layout. Text, headings, order, emoji, tables, badges, lists and HTML are all yours; you place the values.

## Quick start

1. Wrap the part of your README that GitScope may change in a block:

   ```markdown
   ## Statistics

   <!-- gitscope:start -->

   Project started: {first_commit}

   Latest commit: {last_commit}

   Total commits: {commits}

   Contributors: {contributors}

   <!-- gitscope:end -->
   ```

2. Run, at the root of the repository:

   ```bash
   gitscope update
   ```

3. Commit the result. Run `gitscope update` again whenever you want fresh numbers; if nothing changed, the file is not touched.

Everything outside the block is never modified, so the rest of the README stays 100 % yours.

## The three commands

| Command | What it does |
|---|---|
| `gitscope update [FILE]` | Rewrites the blocks of `FILE` **in place** (default: `README.md` at the root of the repository). |
| `gitscope markdown FILE` | Prints `FILE` with the variables resolved. **Never modifies `FILE`.** Use `-o OUT` to write a different file (`-o -` is stdout). |
| `gitscope variables` | Lists every variable with a description. |

`update` and `markdown` accept the usual selection options, which change the numbers: `--repo PATH_OR_URL` (default `.`, so a URL works too), `--since`, `--until`, `--author`, `--branch`, plus `--keep-unknown` and `-v`.

```bash
gitscope update                                     # README.md of the current repository
gitscope update docs/STATS.md --since 2026-01-01
gitscope update README.md --repo ../other-repo
gitscope markdown README.template.md -o README.md   # keep a separate template file
gitscope markdown README.md                         # preview, nothing is written
```

### `update` vs. `markdown`

* **`markdown`** is a pure function: template in, resolved document out. Inside blocks only the blocks are resolved; a file **without any block** is resolved as a whole, so a plain `README.template.md` works too.
* **`update`** needs at least one block and rewrites the file. Because the values replace the variables, the template would be lost and the next `update` would have nothing to refresh. So the first `update` keeps each block's template in an **invisible HTML comment** (it does not show on GitHub) followed by the current values:

  ```markdown
  <!-- gitscope:start -->
  <!-- gitscope:template

  Total commits: {commits}

  -->

  Total commits: 361

  <!-- gitscope:end -->
  ```

  The next `update` re-reads that template and refreshes the values from the current data. Running it twice with the same data gives byte-identical files.

  **To change what a block says, edit the template inside the comment**, then run `update`. Edits made to the visible values are overwritten. To stop using GitScope on a block, run `gitscope markdown FILE -o FILE.new`, which writes the values without the stored templates.

## Variables

Run `gitscope variables` for the current list. Counts use thousands separators (`85,164`), dates are `YYYY-MM-DD` in UTC. When a value does not exist (an empty repository, no recognised language...) it reads `n/a`, and counts read `0`.

| Variable | Value |
|---|---|
| **Repository** | |
| `{repository_name}` | Name of the repository |
| `{first_commit}` | Date of the first commit |
| `{last_commit}` | Date of the last commit |
| `{period}` | `2026-01-03 → 2026-09-18` |
| `{branches}` | Number of branches |
| `{tags}` | Number of tags |
| **Commits** | |
| `{commits}` | Number of commits |
| `{first_commit_message}` | First line of the first commit's message |
| `{last_commit_message}` | First line of the last commit's message |
| `{most_active_hour}` | Hour (UTC) with most commits, e.g. `14:00` |
| **Contributors** | |
| `{contributors}` | Number of contributors |
| `{top_author}` | Contributor with most commits |
| `{top_author_commits}` | Their number of commits |
| `{top_author_percentage}` | Their share, e.g. `54.9` (no `%` sign: write `{top_author_percentage}%`) |
| `{author:NAME:commits}` | Commits of one contributor |
| `{author:NAME:insertions}` | Lines added by one contributor |
| `{author:NAME:deletions}` | Lines removed by one contributor |
| **Files** | |
| `{files}` | Files in the current snapshot |
| `{insertions}` | Total lines added |
| `{deletions}` | Total lines removed |
| **Languages** | |
| `{language_count}` | Number of recognised languages |
| `{top_language}` | Language with most files |

The numbers are exactly those of the normal report (`gitscope` and `--json`), computed with the same definitions; see [How the numbers are defined](../README-EN.md#how-the-numbers-are-defined). In particular they follow your `--since`/`--until`/`--author`/`--branch` filters, except *files* and *languages*, which describe the current snapshot.

### Compound variables: `{author:NAME:FIELD}`

A variable can take arguments separated by `:`. `{author:NAME:commits}` matches a contributor by exact name or email (case-insensitive) and gives their commits; `insertions` and `deletions` work the same way.

```markdown
Alice made {author:Alice:commits} commits: +{author:Alice:insertions} / -{author:Alice:deletions}.
```

An author that does not exist is an error (`no contributor named 'X'`), not a silent `0`.

This is a first step: the same mechanism will carry more specific variables later (see the [roadmap](../README-EN.md#roadmap)).

## Syntax rules

* A variable is `{name}` or `{name:arg:arg}`. The name starts with a letter or `_` and continues with letters, digits or `_`.
* **Anything else with braces is left alone**: JSON (`{"a": 1}`), CSS (`a { color: red }`), `{}`, `{ spaced }`, `{two words}`, `${SHELL_VAR}` and `{{mustache}}`.
* To write a literal `{commits}`, escape it with a backslash: `\{commits}` prints `{commits}`. A backslash before anything that is not a variable is left as it is.
* Variables work everywhere inside the processed region: paragraphs, tables, quotes, lists, headings, HTML attributes, link text.
* Values are inserted **as they are, once**. A commit message that contains `{commits}` shows up as `{commits}`; it is never resolved again. `<!--` and `-->` inside a value are written as `&lt;!--` and `--&gt;`, so a commit message cannot open or close a comment or a block.
* Text inside a value is not escaped for Markdown: a `|` in a commit message inside a table cell will break that table. Keep messages out of tables, or escape them yourself.

## Blocks

* A marker must be **alone on its line** (up to three spaces of indent): `<!-- gitscope:start -->` and `<!-- gitscope:end -->`. Spaces inside the comment are flexible (`<!--gitscope:start-->` works).
* Markers inside fenced code (```` ``` ```` or `~~~`), in inline code, or in indented code are **documentation**, not markers, so your README can explain the feature without triggering it.
* You can have **several blocks**. Each one is independent.
* The names the project had before, `gitstatus` and `gitstats`, are still accepted (`<!-- gitstatus:start -->`), including the template comment written by an older version, which is rewritten with the current name on the next `update`.

## Safety

`update` never leaves a half-written or corrupted README:

* The whole file is read, **validated and rendered in memory first**. Only if everything is fine it is written.
* On **any** error the original file is not modified.
* The write is **atomic**: a temporary file is written next to the README, flushed, and renamed over it, so an interrupted run leaves either the old file or the new one. Permissions are kept and a symlinked README is followed, not replaced.
* If the result equals the current content nothing is written at all.
* Line endings (CRLF or LF), a UTF-8 BOM and a missing final newline are preserved. Lines that GitScope adds use the file's own line ending.
* `markdown -o OUT` refuses to overwrite the template it reads.

What is validated:

| Problem | Message (abridged) |
|---|---|
| Start without end | `line 12: <!-- gitscope:start --> is never closed` |
| End without start | `line 30: found <!-- gitscope:end --> without a matching <!-- gitscope:start -->` |
| Duplicated or nested start | `line 20: <!-- gitscope:start --> found inside the block opened at line 12` |
| No block (for `update`) | `no GitScope block found: wrap the part to update in ...` |
| Unknown variable | `Unknown GitScope variable: {comits} (line 14)` + `Did you mean {commits}?` + the list of available variables |
| Misused variable | `Invalid GitScope variable: {commits:1} (line 9): {commits} takes no arguments` |
| Unclosed stored template | `line 14: the stored template comment ... is never closed` |
| `-->` in a block template (for `update`) | the block cannot be stored inside an HTML comment; use `markdown -o` with a separate template file |

All variable problems of the file are reported together, with line numbers.

### Warnings instead of errors

`--keep-unknown` leaves unknown or misused variables exactly as written, prints a warning for each on stderr, and resolves everything else:

```console
$ gitscope update --keep-unknown
gitscope: warning: left {my_custom_thing} untouched (line 14)
Updated /path/to/README.md (1 block, 5 variables)
```

## Examples

All of these work, because GitScope only replaces the variables:

```markdown
Início: {first_commit}

Início do projeto → {first_commit}

> Project started on {first_commit}

**Projeto iniciado em {first_commit}**

🚀 This project has {commits} commits from {contributors} contributors.

| Metric | Value |
|---|---:|
| Commits | {commits} |
| Contributors | {contributors} |
| Files | {files} |

<p align="center"><b>{commits}</b> commits · <b>{contributors}</b> contributors</p>

```

### Keeping a separate template file

```bash
gitscope markdown README.template.md -o README.md
```

`README.template.md` keeps the variables forever; `README.md` is generated and contains only values. There are no blocks needed, since a file without blocks is resolved as a whole.

## Extending it

Adding a variable is one entry in `standard_variables()` in [`src/output/variables.rs`](../src/output/variables.rs): a name, a group, a description and a resolver function. Resolvers receive the arguments written after the name, so compound variables use the same mechanism. Nothing else needs to change; `gitscope variables`, the error messages and the suggestions pick the new variable up automatically. See [ARCHITECTURE-EN.md](ARCHITECTURE-EN.md).
