# JSON output

[Português](JSON.md) · **English**

`gitscope <PATH> --json` prints one JSON document to **stdout**. Diagnostics (`--verbose`) and errors go to stderr, so the output can always be redirected or piped:

```bash
gitscope ./repo --json > stats.json
gitscope ./repo --json | jq '.contributors[] | {name, commits}'
```

The document is pretty-printed, UTF-8, and ends with a newline. Terminal-only options such as `--top` do **not** affect it: lists are never truncated.

## Conventions

- **Dates and times** are RFC 3339 strings in UTC, e.g. `"2026-01-03T12:30:00Z"`. Plain calendar days (in `filters`) are `"YYYY-MM-DD"`. Months are `"YYYY-MM"`.
- **Counts** are non-negative integers. **Percentages** are numbers between 0 and 100, rounded to two decimals.
- **Absent vs. `null`.** A field is `null` when it exists but has no value (for example `commits.first` in an empty repository). The only field that can be *absent* is `commit_details` (see below), plus `old_path` inside a change.
- **Ordering.** Every array has a deterministic order, documented per field, so two runs over the same repository produce identical output.
- **Buckets are UTC.** Weekdays, hours and months are computed from author dates in UTC.

## Top-level object

| Field | Type | Description |
|---|---|---|
| `schema_version` | integer | Version of this schema. Currently `1`. |
| `repository` | object | Repository-level facts. |
| `filters` | object | The filters that were applied. |
| `commits` | object | Commit statistics for the filtered history. |
| `contributors` | array | One entry per author identity. |
| `files` | object | File statistics. |
| `languages` | array | Language breakdown of the file snapshot. |
| `activity` | object | Commits over time. |
| `commit_details` | array | **Only present with `--all`.** Every matching commit in detail. |

## `repository`

| Field | Type | Description |
|---|---|---|
| `name` | string | Directory name of the repository (a `.git` suffix is dropped). |
| `branches` | integer | Number of **local** branches; when a remote URL was analysed, the number of branches of the remote. Independent of the filters. |
| `tags` | integer | Number of tags, lightweight and annotated. Independent of the filters. |

## `filters`

Echo of the options given. Unused filters are `null`.

| Field | Type | Description |
|---|---|---|
| `since` | string \| null | `--since`, as `YYYY-MM-DD`. |
| `until` | string \| null | `--until`, as `YYYY-MM-DD`. |
| `author` | string \| null | `--author`, exactly as given. |
| `branch` | string \| null | `--branch`, exactly as given. |

## `commits`

| Field | Type | Description |
|---|---|---|
| `total` | integer | Commits that passed the filters (merge commits included). |
| `first` | string \| null | Author date of the oldest matching commit. |
| `last` | string \| null | Author date of the newest matching commit. |
| `first_message` | string \| null | First line of the message of the oldest matching commit. |
| `last_message` | string \| null | First line of the message of the newest matching commit. |
| `insertions` | integer | Lines added by all matching commits (the sum of the contributors' `insertions`). |
| `deletions` | integer | Lines removed by all matching commits. |
| `most_active_hour` | integer \| null | Hour of day (0–23, UTC) with most commits. The earliest hour wins a tie. `null` when there are no commits. |
| `by_weekday` | array | Always **7** entries, Monday first: `{ "weekday": "Mon" \| "Tue" \| … \| "Sun", "commits": integer }`. |
| `by_hour` | array | Always **24** entries, hour 0 first: `{ "hour": 0–23, "commits": integer }`. |

## `contributors[]`

One entry per author **identity**, identified by the lowercase email. Sorted by `commits` descending, then by `name` (case-insensitive), then by `email`.

| Field | Type | Description |
|---|---|---|
| `name` | string | Name used in the identity's most recent commit. |
| `email` | string | Email as written in that commit. |
| `commits` | integer | Number of commits. |
| `percentage` | number | Share of all commits that passed the filters. |
| `files_changed` | integer | Sum, over the contributor's commits, of files touched. A file touched in two commits counts twice. |
| `insertions` | integer | Lines added. |
| `deletions` | integer | Lines removed. |
| `first_commit` | string | Author date of the first commit. |
| `last_commit` | string | Author date of the last commit. |

Merge commits count in `commits` but contribute no files or lines.

## `files`

| Field | Type | Description |
|---|---|---|
| `total` | integer | Number of files in the **snapshot**: the tree at the tip of the analysed history (`HEAD` or `--branch`). Not affected by `--since`, `--until` or `--author`. |
| `extensions` | array | `{ "extension": string \| null, "count": integer }` over the snapshot, sorted by `count` descending, then by `extension`. `extension` is lowercase without the dot; `null` means the file has no extension. |
| `most_modified` | array | `{ "path": string, "modifications": integer }`, sorted by `modifications` descending, then by `path`. Computed from the **filtered history**, so it can include files that no longer exist. |

A file is modified once for every filtered commit in which it appears as changed (added, modified, deleted, or the destination of a rename).

## `languages[]`

Derived from the extensions of the snapshot. Sorted by `files` descending, then by `name`. Only files with a recognised language are counted, so `percentage` values add up to 100 (up to rounding). The list is empty when nothing is recognised.

| Field | Type | Description |
|---|---|---|
| `name` | string | Language name, e.g. `"Rust"`, `"TypeScript"`. |
| `files` | integer | Files attributed to the language. |
| `percentage` | number | Share among files with a recognised language. |

## `activity`

| Field | Type | Description |
|---|---|---|
| `by_month` | array | `{ "month": "YYYY-MM", "commits": integer }`, chronological, from the first to the last active month. Months without commits are included with `0`. Empty when there are no commits. |

## `commit_details[]` (only with `--all`)

Newest commit first (by author date; ties keep history order). When `--all` is used the key is always present, even if empty.

| Field | Type | Description |
|---|---|---|
| `hash` | string | Full 40-character object id. |
| `author` | string | Author name. |
| `email` | string | Author email. |
| `date` | string | Author date, UTC. |
| `message` | string | First line of the commit message. |
| `files_changed` | integer | Number of entries in `changes`. |
| `insertions` | integer | Lines added. |
| `deletions` | integer | Lines removed. |
| `changes` | array | Files touched, sorted by `path`. Empty for merge commits and empty commits. |

Each entry of `changes`:

| Field | Type | Description |
|---|---|---|
| `path` | string | Path after the commit (path before it, for deletions). |
| `old_path` | string | **Absent** unless `kind` is `"renamed"`: the previous path. |
| `kind` | string | `"added"`, `"modified"`, `"deleted"` or `"renamed"`. |

## Example with `--all`

```bash
gitscope ./repo --author Bob --all --json
```

```json
{
  "schema_version": 1,
  "repository": { "name": "my-project", "branches": 3, "tags": 2 },
  "filters": { "since": null, "until": null, "author": "Bob", "branch": null },
  "commits": { "total": 35, "...": "..." },
  "contributors": [ { "name": "Bob", "email": "bob@example.com", "commits": 35, "percentage": 100.0, "...": "..." } ],
  "files": { "...": "..." },
  "languages": [ "..." ],
  "activity": { "by_month": [ "..." ] },
  "commit_details": [
    {
      "hash": "ae00d6a031eff59a79fcc1de177d57d25680329c",
      "author": "Bob",
      "email": "bob@example.com",
      "date": "2026-04-26T09:38:00Z",
      "message": "fix: handle empty repositories",
      "files_changed": 2,
      "insertions": 10,
      "deletions": 3,
      "changes": [
        { "path": "docs/README.md", "old_path": "README.md", "kind": "renamed" },
        { "path": "src/main.rs", "kind": "modified" }
      ]
    }
  ]
}
```

(`"..."` marks parts omitted for brevity.)

## Empty repositories and empty results

The document keeps its shape:

```json
{
  "commits": { "total": 0, "first": null, "last": null, "first_message": null, "last_message": null, "insertions": 0, "deletions": 0, "most_active_hour": null, "by_weekday": [ "7 entries" ], "by_hour": [ "24 entries" ] },
  "contributors": [],
  "activity": { "by_month": [] }
}
```

(Other fields omitted.) `commit_details` is `[]` when `--all` was requested.

## Stability policy

`schema_version` identifies the schema described here.

- **Non-breaking changes**, which keep the version: adding new fields or new values of documented enums. **Consumers must ignore fields they do not know.**
- **Breaking changes**, which bump `schema_version`: removing or renaming a field, changing a field's type or meaning, or changing an ordering guarantee.

The repository's test suite pins the exact set of keys of every object, so an accidental schema change fails the build.

## `jq` recipes

```bash
# Top 3 contributors by commits
gitscope . --json | jq -r '.contributors[:3][] | "\(.name)\t\(.commits)"'

# Commits per month as CSV
gitscope . --json | jq -r '.activity.by_month[] | [.month, .commits] | @csv'

# Files touched by one author, most changed first
gitscope . --author alice --json | jq -r '.files.most_modified[] | "\(.modifications)\t\(.path)"'

# Every commit of an author, one per line
gitscope . --author alice --all --json | jq -r '.commit_details[] | "\(.date) \(.hash[0:7]) \(.message)"'
```
