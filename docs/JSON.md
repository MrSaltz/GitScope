# Saída JSON

**Português** · [English](JSON-EN.md)

`gitscope <PATH> --json` imprime um documento JSON no **stdout**. Diagnósticos (`--verbose`) e erros vão para o stderr, então a saída sempre pode ser redirecionada ou encadeada num pipe:

```bash
gitscope ./repo --json > stats.json
gitscope ./repo --json | jq '.contributors[] | {name, commits}'
```

O documento é indentado, em UTF-8 e termina com uma quebra de linha. Opções que só valem para o terminal, como `--top`, **não** o afetam: as listas nunca são truncadas.

## Convenções

- **Datas e horas** são strings RFC 3339 em UTC, por exemplo `"2026-01-03T12:30:00Z"`. Dias simples (em `filters`) são `"YYYY-MM-DD"`. Meses são `"YYYY-MM"`.
- **Contagens** são inteiros não negativos. **Percentuais** são números entre 0 e 100, com duas casas decimais.
- **Ausente vs. `null`.** Um campo é `null` quando existe mas não tem valor (por exemplo, `commits.first` num repositório vazio). O único campo que pode estar *ausente* é `commit_details` (veja abaixo), além de `old_path` dentro de uma mudança.
- **Ordem.** Toda lista tem uma ordem determinística, documentada por campo, então duas execuções sobre o mesmo repositório geram saídas idênticas.
- **Faixas em UTC.** Dias da semana, horas e meses são calculados a partir das datas do autor em UTC.

## Objeto raiz

| Campo | Tipo | Descrição |
|---|---|---|
| `schema_version` | inteiro | Versão deste schema. Atualmente `1`. |
| `repository` | objeto | Fatos do repositório. |
| `filters` | objeto | Os filtros que foram aplicados. |
| `commits` | objeto | Estatísticas dos commits do histórico filtrado. |
| `contributors` | lista | Um item por identidade de autor. |
| `files` | objeto | Estatísticas de arquivos. |
| `languages` | lista | Distribuição de linguagens do retrato dos arquivos. |
| `activity` | objeto | Commits ao longo do tempo. |
| `commit_details` | lista | **Só presente com `--all`.** Todos os commits que casam, em detalhe. |

## `repository`

| Campo | Tipo | Descrição |
|---|---|---|
| `name` | string | Nome do diretório do repositório (o sufixo `.git` é removido). |
| `branches` | inteiro | Número de branches **locais**; ao analisar uma URL remota, o número de branches do remoto. Independe dos filtros. |
| `tags` | inteiro | Número de tags, leves e anotadas. Independe dos filtros. |

## `filters`

Eco das opções dadas. Filtros não usados são `null`.

| Campo | Tipo | Descrição |
|---|---|---|
| `since` | string \| null | `--since`, como `YYYY-MM-DD`. |
| `until` | string \| null | `--until`, como `YYYY-MM-DD`. |
| `author` | string \| null | `--author`, exatamente como dado. |
| `branch` | string \| null | `--branch`, exatamente como dado. |

## `commits`

| Campo | Tipo | Descrição |
|---|---|---|
| `total` | inteiro | Commits que passaram nos filtros (merges incluídos). |
| `first` | string \| null | Data do autor do commit mais antigo que casa. |
| `last` | string \| null | Data do autor do commit mais recente que casa. |
| `first_message` | string \| null | Primeira linha da mensagem do commit mais antigo que casa. |
| `last_message` | string \| null | Primeira linha da mensagem do commit mais recente que casa. |
| `insertions` | inteiro | Linhas adicionadas por todos os commits que casam (a soma das `insertions` dos contribuidores). |
| `deletions` | inteiro | Linhas removidas por todos os commits que casam. |
| `most_active_hour` | inteiro \| null | Hora do dia (0–23, UTC) com mais commits. Em empate, vence a mais cedo. `null` quando não há commits. |
| `by_weekday` | lista | Sempre **7** itens, começando na segunda: `{ "weekday": "Mon" \| "Tue" \| … \| "Sun", "commits": inteiro }`. |
| `by_hour` | lista | Sempre **24** itens, começando na hora 0: `{ "hour": 0–23, "commits": inteiro }`. |

## `contributors[]`

Um item por **identidade** de autor, identificada pelo e-mail em minúsculas. Ordenado por `commits` decrescente, depois por `name` (sem diferenciar maiúsculas) e por `email`.

| Campo | Tipo | Descrição |
|---|---|---|
| `name` | string | Nome usado no commit mais recente da identidade. |
| `email` | string | E-mail como escrito nesse commit. |
| `commits` | inteiro | Número de commits. |
| `percentage` | número | Parcela de todos os commits que passaram nos filtros. |
| `files_changed` | inteiro | Soma, sobre os commits do contribuidor, dos arquivos tocados. Um arquivo tocado em dois commits conta duas vezes. |
| `insertions` | inteiro | Linhas adicionadas. |
| `deletions` | inteiro | Linhas removidas. |
| `first_commit` | string | Data do autor do primeiro commit. |
| `last_commit` | string | Data do autor do último commit. |

Merges contam em `commits`, mas não contribuem com arquivos nem linhas.

## `files`

| Campo | Tipo | Descrição |
|---|---|---|
| `total` | inteiro | Número de arquivos no **retrato**: a árvore na ponta do histórico analisado (`HEAD` ou `--branch`). Não é afetado por `--since`, `--until` nem `--author`. |
| `extensions` | lista | `{ "extension": string \| null, "count": inteiro }` sobre o retrato, ordenada por `count` decrescente e depois por `extension`. `extension` é minúscula, sem o ponto; `null` significa que o arquivo não tem extensão. |
| `most_modified` | lista | `{ "path": string, "modifications": inteiro }`, ordenada por `modifications` decrescente e depois por `path`. Calculada a partir do **histórico filtrado**, então pode incluir arquivos que não existem mais. |

Um arquivo é modificado uma vez por commit filtrado em que aparece alterado (criado, modificado, apagado ou destino de um rename).

## `languages[]`

Derivada das extensões do retrato. Ordenada por `files` decrescente e depois por `name`. Só arquivos com linguagem reconhecida contam, então os valores de `percentage` somam 100 (a menos de arredondamento). A lista fica vazia quando nada é reconhecido.

| Campo | Tipo | Descrição |
|---|---|---|
| `name` | string | Nome da linguagem, por exemplo `"Rust"`, `"TypeScript"`. |
| `files` | inteiro | Arquivos atribuídos à linguagem. |
| `percentage` | número | Parcela entre os arquivos com linguagem reconhecida. |

## `activity`

| Campo | Tipo | Descrição |
|---|---|---|
| `by_month` | lista | `{ "month": "YYYY-MM", "commits": inteiro }`, em ordem cronológica, do primeiro ao último mês com atividade. Meses sem commits entram com `0`. Vazia quando não há commits. |

## `commit_details[]` (só com `--all`)

Do commit mais novo ao mais antigo (pela data do autor; empates mantêm a ordem do histórico). Quando `--all` é usado, a chave sempre está presente, mesmo vazia.

| Campo | Tipo | Descrição |
|---|---|---|
| `hash` | string | Id completo do objeto, com 40 caracteres. |
| `author` | string | Nome do autor. |
| `email` | string | E-mail do autor. |
| `date` | string | Data do autor, em UTC. |
| `message` | string | Primeira linha da mensagem do commit. |
| `files_changed` | inteiro | Número de itens em `changes`. |
| `insertions` | inteiro | Linhas adicionadas. |
| `deletions` | inteiro | Linhas removidas. |
| `changes` | lista | Arquivos tocados, ordenados por `path`. Vazia em merges e em commits vazios. |

Cada item de `changes`:

| Campo | Tipo | Descrição |
|---|---|---|
| `path` | string | Caminho depois do commit (o caminho de antes, em exclusões). |
| `old_path` | string | **Ausente**, a menos que `kind` seja `"renamed"`: o caminho anterior. |
| `kind` | string | `"added"`, `"modified"`, `"deleted"` ou `"renamed"`. |

## Exemplo com `--all`

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

(`"..."` marca trechos omitidos por brevidade.)

## Repositórios vazios e resultados vazios

O documento mantém o formato:

```json
{
  "commits": { "total": 0, "first": null, "last": null, "first_message": null, "last_message": null, "insertions": 0, "deletions": 0, "most_active_hour": null, "by_weekday": [ "7 entries" ], "by_hour": [ "24 entries" ] },
  "contributors": [],
  "activity": { "by_month": [] }
}
```

(Demais campos omitidos.) `commit_details` é `[]` quando `--all` foi pedido.

## Política de estabilidade

`schema_version` identifica o schema descrito aqui.

- **Mudanças compatíveis**, que mantêm a versão: acrescentar campos novos ou novos valores de enums documentados. **Quem consome deve ignorar os campos que não conhece.**
- **Mudanças incompatíveis**, que aumentam `schema_version`: remover ou renomear um campo, mudar o tipo ou o significado de um campo, ou mudar uma garantia de ordem.

A suíte de testes do repositório trava o conjunto exato de chaves de cada objeto, então uma mudança acidental no schema quebra o build.

## Receitas de `jq`

```bash
# Os 3 principais contribuidores por commits
gitscope . --json | jq -r '.contributors[:3][] | "\(.name)\t\(.commits)"'

# Commits por mês em CSV
gitscope . --json | jq -r '.activity.by_month[] | [.month, .commits] | @csv'

# Arquivos tocados por um autor, os mais alterados primeiro
gitscope . --author alice --json | jq -r '.files.most_modified[] | "\(.modifications)\t\(.path)"'

# Todos os commits de um autor, um por linha
gitscope . --author alice --all --json | jq -r '.commit_details[] | "\(.date) \(.hash[0:7]) \(.message)"'
```
