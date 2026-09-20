# gitscope

**Português** · [English](README-EN.md)

**Uma CLI em Rust para analisar estatísticas de repositórios Git.**

Aponte o `gitscope` para um repositório e veja commits, contribuidores,
arquivos, linguagens e atividade, num relatório legível no terminal ou em JSON.

- Lê o repositório direto com o [`git2`](https://crates.io/crates/git2) (libgit2). **Não precisa do executável `git`.**
- Funciona com um diretório de trabalho, um diretório `.git` ou um repositório bare.
- Filtra por autor, período e branch; os filtros se combinam.
- Inspeciona todos os commits de um autor, com os arquivos que cada um tocou e as linhas alteradas.
- Saída JSON estável e documentada, para scripts e dashboards.
- Coloque os números no seu próprio README: escreva `{commits}`, `{contributors}`... em Markdown e deixe o `gitscope update` preenchê-los.
- Analisa um **repositório remoto** direto pela URL: ele é clonado num diretório temporário que é apagado depois.
- Somente leitura e local por padrão: a rede só é usada quando você passa uma URL. Sem servidor, banco de dados, contas nem telemetria.

## Exemplo

```console
$ gitscope ./my-project --top 5
```

```text
╭──────────────────────────────────────────────────────────╮
│                     GITSCOPE v1.0.0                      │
│                        my-project                        │
╰──────────────────────────────────────────────────────────╯

Repository
────────────────────────────────────────────────────────────
  Commits                                  102
  Contributors                               3
  Branches                                   3
  Tags                                       2
  Period               2026-01-02 → 2026-04-26

Commits
────────────────────────────────────────────────────────────
  Total                                    102
  First commit                      2026-01-02
  Last commit                       2026-04-26
  Most active hour                       17:00

  By weekday
  Mon  █████████████████████████████  19
  Tue  ██████████████                 9
  Wed  █████████████████████          14
  Thu  ███████████████████████        15
  Fri  ████████████████████           13
  Sat  ██████████████████             12
  Sun  ██████████████████████████████ 20

  By hour (UTC)
  ▂▄▂······▇▄▄▄▅▆▅██▆▁▂▁▂▁
  0     6     12    18

Top contributors
────────────────────────────────────────────────────────────
  Alice                56 commits  54.9%      +770      -72
  Bob                  35 commits  34.3%      +457      -65
  Charlie              11 commits  10.8%      +141       -4

Files
────────────────────────────────────────────────────────────
  Total files                               10

  Extensions
  .rs               4
  .md               1
  .py               1
  .sh               1
  .toml             1
  … and 2 more (raise --top to see them)

  Most modified files
  Cargo.toml                                           15
  src/lib.rs                                           13
  tools/report.py                                      12
  src/stats.rs                                         11
  web/app.ts                                           11
  … and 5 more (raise --top to see them)

Languages
────────────────────────────────────────────────────────────
  Rust            50.0%  ██████████
  TypeScript      25.0%  █████
  Python          12.5%  ███
  Shell           12.5%  ███

Activity
────────────────────────────────────────────────────────────
  2026-01  █████████████████              19
  2026-02  ███████████████████            21
  2026-03  ██████████████████████████     29
  2026-04  ██████████████████████████████ 33
```

*(Saída real; a saída do programa é em inglês. Para renderizar, o terminal precisa suportar UTF-8.)*

## Instalação

### Binários prontos

Cada release no GitHub (a página *Releases*) tem um arquivo por plataforma e um arquivo `SHA256SUMS`:

| Plataforma | Arquivo |
|---|---|
| Linux (x86_64) | `gitscope-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Windows (x86_64) | `gitscope-vX.Y.Z-x86_64-pc-windows-msvc.zip` |
| macOS (Apple Silicon) | `gitscope-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `gitscope-vX.Y.Z-x86_64-apple-darwin.tar.gz` |

Extraia, ponha o `gitscope` (`gitscope.exe` no Windows) numa pasta que esteja no seu `PATH` e confira com `gitscope --version`. Para verificar o download:

```bash
sha256sum -c SHA256SUMS --ignore-missing                                       # Linux
shasum -a 256 -c SHA256SUMS --ignore-missing                                   # macOS
Get-FileHash .\gitscope-vX.Y.Z-x86_64-pc-windows-msvc.zip -Algorithm SHA256    # Windows (PowerShell): compare com o SHA256SUMS
```

Os binários do Linux e do macOS levam o OpenSSL dentro, então não dependem da libssl do sistema. Os binários aparecem aqui quando a primeira release for publicada; até lá, compile a partir do código-fonte.

### A partir do código-fonte

Requisitos:

- [Rust](https://rustup.rs) 1.87 ou mais novo.
- Um compilador C, porque o `git2` compila a libgit2 do código-fonte: as *MSVC Build Tools* no Windows, `gcc`/`clang` no Linux, as Xcode Command Line Tools no macOS.
- No **Linux** e no **macOS**, o OpenSSL e o `pkg-config` (para clonar por HTTPS): `sudo apt install pkg-config libssl-dev` no Debian/Ubuntu, `sudo dnf install pkgconf-pkg-config openssl-devel` no Fedora, `brew install openssl pkg-config` no macOS. O Windows usa a pilha TLS do sistema e não precisa de nada extra.

Sem OpenSSL, ou sem precisar de repositórios remotos? Gere um binário só local com `cargo install --path . --no-default-features` (ou embuta o OpenSSL com `--features vendored-openssl`, que precisa de `perl` e `make`).

```bash
git clone https://github.com/MrSaltz/GitScope.git
cd gitscope
cargo install --path .
```

Ou, sem clonar:

```bash
cargo install --git https://github.com/MrSaltz/GitScope.git
```

Confira a instalação:

```bash
gitscope --version
```

### Compilar sem instalar

```bash
cargo build --release
./target/release/gitscope --help      # target\release\gitscope.exe no Windows
```

## Uso

```text
gitscope [OPTIONS] [PATH_OR_URL]      imprime o relatório (padrão)
gitscope update [FILE]                atualiza as {variáveis} de um README no lugar
gitscope markdown FILE                imprime um template Markdown com as variáveis resolvidas
gitscope variables                    lista as variáveis
gitscope tui [PATH_OR_URL]            navega pelas estatísticas numa interface interativa
```

`PATH_OR_URL` é o repositório a analisar e, por padrão, é o diretório atual. Pode ser um diretório de trabalho, um diretório `.git`, um repositório bare ou a URL de um [repositório remoto](#repositórios-remotos). Os diretórios pais não são procurados.

```bash
gitscope                         # analisa o diretório atual
gitscope /caminho/do/repositorio # analisa outro repositório
gitscope ./repo/.git             # o diretório .git também serve
gitscope https://github.com/owner/repo.git   # um repositório remoto
```

### Opções

| Opção | Descrição |
|---|---|
| `--since <DATE>` | Só commits escritos a partir deste dia (`YYYY-MM-DD`, inclusive). |
| `--until <DATE>` | Só commits escritos até este dia (`YYYY-MM-DD`, inclusive). |
| `--author <AUTHOR>` | Só commits cujo **nome ou e-mail do autor contém** este texto (sem diferenciar maiúsculas). Ativa a [visão do autor](#análise-de-um-autor). |
| `--branch <BRANCH>` | Analisa esta branch no lugar do `HEAD`. Tenta as locais primeiro e depois as remotas, como `origin/main`. |
| `--json` | Imprime o relatório em JSON em vez de texto formatado. Veja [Saída JSON](#saída-json). |
| `--all` | Lista **todos os commits** que passam nos filtros, com detalhes. Os detalhes de cada commit só são coletados com esta opção. |
| `-v`, `--verbose` | Imprime diagnósticos no **stderr**. Junto com `--all`, imprime o bloco completo de cada commit. |
| `--top <N>` | Máximo de linhas nas listas ordenadas da saída do terminal (padrão `10`). O JSON nunca é truncado. |
| `-h`, `--help` / `-V`, `--version` | Ajuda e versão. |

Os filtros se combinam: um commit precisa satisfazer todos eles.

### Códigos de saída e fluxos

| Código | Significado |
|---|---|
| `0` | Sucesso (inclusive repositório vazio ou filtros que não encontram nada). |
| `1` | Erro de execução: caminho inexistente, não é um repositório Git, branch desconhecida, `--since` depois de `--until`, URL não suportada, falha no clone, falha da libgit2. |
| `2` | Linha de comando inválida (por exemplo, uma data malformada). |

O relatório vai para o **stdout**; erros e diagnósticos do `--verbose` vão para o **stderr**, então `gitscope ./repo --json > stats.json` sempre gera um arquivo limpo.

## Exemplos

Filtrar por período:

```bash
gitscope ./repo --since 2026-01-01 --until 2026-06-30
```

Analisar uma branch:

```bash
gitscope ./repo --branch main
```

Tudo de uma vez:

```bash
gitscope ./repo \
  --author "Widison" \
  --branch main \
  --since 2026-01-01 \
  --until 2026-09-01 \
  --all
```

Passar por um paginador ou exportar:

```bash
gitscope ./repo | less
gitscope ./repo --json > stats.json
```

### Análise de um autor

```bash
gitscope ./my-project --author Alice --top 3
```

```text
╭──────────────────────────────────────────────────────────╮
│                     GITSCOPE v1.0.0                      │
│                        my-project                        │
╰──────────────────────────────────────────────────────────╯

Filters: author "Alice"

Contributor
────────────────────────────────────────────────────────────
  Name                                   Alice
  Email                      alice@example.com
  Commits                                   56
  Files changed                             56
  Insertions                              +770
  Deletions                                -72
  First commit                      2026-01-02
  Last commit                       2026-04-26
...
```

Se o texto casar com várias identidades (por exemplo, dois e-mails), um bloco *Contributor* é impresso para cada uma.

### Inspecionando os commits de um autor (`--all`)

```bash
gitscope ./my-project --author Bob --all --since 2026-04-20
```

```text
Commits by Bob
────────────────────────────────────────────────────────────

2026-04-26  09:38  ae00d6a
fix: handle empty repositories
```

Acrescente `--verbose` para ver o bloco completo de cada commit, com cada arquivo tocado (`+` criado, `M` modificado, `-` apagado, `R` renomeado):

```bash
gitscope ./my-project --author Bob --all --verbose --since 2026-04-24
```

```text
Commits by Bob
────────────────────────────────────────────────────────────

ae00d6a
────────────────────────────────────────
Author       Bob <bob@example.com>
Date         2026-04-26 09:38
Message      fix: handle empty repositories

Changes
  M src/main.rs

Files changed      1
Insertions       +10
Deletions         -0
```

`--all` também funciona sem `--author`: lista todos os commits que passam nos outros filtros.

## Repositórios remotos

Passe uma URL no lugar de um caminho e o `gitscope` clona o repositório num diretório temporário, analisa e apaga o clone. Todas as opções e filtros funcionam normalmente:

```bash
gitscope https://github.com/owner/repo.git
gitscope https://github.com/owner/repo.git --since 2026-01-01 --author alice --all
gitscope https://github.com/owner/repo.git --branch develop --json > stats.json
```

Enquanto trabalha, o stderr mostra:

```text
gitscope: cloning https://github.com/owner/repo.git into a temporary directory...
  Receiving objects:  45% (1234/2741), 3.2 MiB      <- progresso ao vivo, só em terminal
```

Como se comporta:

- **URLs suportadas:** `https://`, `http://`, `git://` e `file://`. Um caminho local que exista sempre vence a sintaxe de URL.
- **Só repositórios públicos.** URLs SSH (`ssh://…`, `git@host:owner/repo`) são recusadas com uma explicação, porque autenticação está fora do escopo. Um repositório privado, ou que não existe (os hosts respondem `401` nos dois casos), falha com uma mensagem dizendo isso; não há pedido de credenciais.
- **Credenciais nunca são impressas.** Se você puser um token numa URL (`https://token@host/…`), ele é trocado por `***` em toda mensagem.
- **Histórico completo.** As estatísticas precisam de todos os commits, então o histórico inteiro é baixado: espere de alguns segundos a minutos em repositórios grandes. O clone é *bare*: sem árvore de trabalho, sem arquivos extraídos e sem executar hooks.
- **Limpeza.** O diretório temporário (`gitscope-XXXXXX` dentro do diretório temporário do sistema) é removido quando a análise termina, inclusive se ela falhar. A mensagem do clone, o progresso e os diagnósticos vão para o **stderr**, então o `--json` no stdout continua limpo. Se o processo for morto (por exemplo, com Ctrl-C), o sistema operacional não deixa ele limpar, e o diretório `gitscope-*` pode ficar para trás; é seguro apagá-lo.
- **Branches.** Num clone, *branches* conta as branches do remoto, e `--branch` aceita uma branch remota pelo nome simples (`--branch develop`) ou como `origin/develop`.
- **Validação primeiro.** Argumentos ruins (por exemplo, `--since` depois de `--until`) são reportados antes de qualquer coisa ser baixada.

## Templates Markdown

Coloque estatísticas no seu próprio README. Escreva o Markdown que quiser e ponha **variáveis** onde os números devem ficar:

```markdown
## Statistics

<!-- gitscope:start -->

Project started: {first_commit}

Latest commit: {last_commit}

Total commits: {commits}

Contributors: {contributors}

<!-- gitscope:end -->
```

```bash
gitscope update
```

```markdown
<!-- gitscope:start -->
<!-- gitscope:template

Project started: {first_commit}
...
-->

Project started: 2026-01-02

Latest commit: 2026-04-26

Total commits: 102

Contributors: 3

<!-- gitscope:end -->
```

O GitScope troca **só as variáveis**, e só dentro dos blocos: todo o resto do README fica byte a byte como você escreveu. Ele não impõe layout, então uma frase, uma tabela, uma citação, um badge ou HTML funcionam:

```markdown
🚀 This project has {commits} commits from {contributors} contributors.

| Metric | Value |
|---|---:|
| Commits | {commits} |
| Files | {files} |
```

| Comando | O que faz |
|---|---|
| `gitscope update [FILE]` | Atualiza os blocos de `FILE` (padrão: `README.md` na raiz do repositório), no lugar e de forma atômica. |
| `gitscope markdown FILE` | Imprime `FILE` com as variáveis resolvidas; nunca o modifica (`-o OUT` grava outro arquivo). |
| `gitscope variables` | Lista as variáveis: `{commits}`, `{contributors}`, `{first_commit}`, `{top_author}`, `{author:NAME:commits}`... |

O comentário invisível `<!-- gitscope:template -->` é como o bloco se lembra do seu template, para o próximo `gitscope update` poder renovar os números (repetir com os mesmos dados não muda nada). Variável desconhecida é um erro que lista as disponíveis; um bloco malformado ou qualquer erro deixa o arquivo intocado.

Guia completo, lista de variáveis, sintaxe, garantias de segurança e exemplos: **[docs/MARKDOWN.md](docs/MARKDOWN.md)**.

## Interface interativa (TUI)

```bash
gitscope tui                                   # o diretório atual
gitscope tui ./repo --since 2026-01-01
gitscope tui https://github.com/owner/repo.git
```

Navegue pelas mesmas estatísticas com o teclado, em seis abas: **Overview**, **Contributors**, **Files**, **Activity**, **Commits** e **Configurations**. O repositório é analisado uma vez, antes de a interface abrir (os filtros de sempre, `--since`, `--until`, `--author` e `--branch`, e as URLs funcionam), e a interface só navega pelo resultado. A lista de commits pode ser filtrada enquanto você digita, por autor, e-mail, mensagem ou hash, e mostra os arquivos, as inserções e as deleções do commit selecionado.

| Tecla | Ação |
|---|---|
| `←` `→`, `Tab`, `h` `l`, `1`…`6` | Troca de aba |
| `↑` `↓`, `k` `j`, `PgUp` `PgDn`, `Home` `End` | Move a seleção |
| `Enter` (`Espaço` também) | Em *Contributors*: abre os commits daquela pessoa; em *Configurations*: muda a configuração |
| `/` | Filtra os commits; `Enter` mantém o filtro, `Esc` o limpa |
| `?` | Ajuda |
| `q`, `Ctrl+C` | Sai |

Precisa de um terminal de verdade (stdin e stdout); para pipes e arquivos use o relatório normal ou `--json`. A lista de commits guarda o detalhe de todos os commits na memória, então em históricos com centenas de milhares de commits ela é mais pesada que o relatório normal. A interface existe em inglês e em português (veja abaixo).

### Configuração

A aba **Configurations** (tecla `6`) muda o idioma da interface: **English** ou **Português (Brasil)**. Selecione a configuração e pressione `Enter` ou `Espaço`; a tela é traduzida na hora e a escolha é guardada para a próxima vez, num pequeno arquivo JSON:

```json
{ "language": "pt" }
```

| Sistema | Arquivo de configuração |
|---|---|
| Windows | `%APPDATA%\gitscope\config.json` |
| macOS | `~/Library/Application Support/gitscope/config.json` |
| Linux | `$XDG_CONFIG_HOME/gitscope/config.json` (ou `~/.config/gitscope/config.json`) |

Defina `GITSCOPE_CONFIG` para usar outro arquivo. Só a interface é traduzida: o relatório, as mensagens e os outros comandos continuam em inglês. Um arquivo de configuração ilegível é ignorado com um aviso e reescrito na próxima vez que você mudar uma configuração.

## Saída JSON

```bash
gitscope ./my-project --json > stats.json
```

```json
{
  "schema_version": 1,
  "repository": { "name": "my-project", "branches": 3, "tags": 2 },
  "filters": { "since": null, "until": null, "author": null, "branch": null },
  "commits": {
    "total": 102,
    "first": "2026-01-02T11:50:00Z",
    "last": "2026-04-26T14:44:00Z",
    "most_active_hour": 17,
    "by_weekday": [ { "weekday": "Mon", "commits": 19 }, "..." ],
    "by_hour": [ { "hour": 0, "commits": 2 }, "..." ]
  },
  "contributors": [
    {
      "name": "Alice",
      "email": "alice@example.com",
      "commits": 56,
      "percentage": 54.9,
      "files_changed": 56,
      "insertions": 770,
      "deletions": 72,
      "first_commit": "2026-01-02T11:50:00Z",
      "last_commit": "2026-04-26T14:44:00Z"
    }
  ],
  "files": {
    "total": 10,
    "extensions": [ { "extension": "rs", "count": 4 }, "..." ],
    "most_modified": [ { "path": "Cargo.toml", "modifications": 15 }, "..." ]
  },
  "languages": [ { "name": "Rust", "files": 4, "percentage": 50.0 }, "..." ],
  "activity": { "by_month": [ { "month": "2026-01", "commits": 19 }, "..." ] }
}
```

*(Listas abreviadas com `"..."` neste README.)*

Com `--all`, é acrescentada uma lista `commit_details`, do commit mais novo ao mais antigo:

```bash
gitscope ./my-project --author Bob --all --json --since 2026-04-24
```

```json
"commit_details": [
  {
    "hash": "ae00d6a031eff59a79fcc1de177d57d25680329c",
    "author": "Bob",
    "email": "bob@example.com",
    "date": "2026-04-26T09:38:00Z",
    "message": "fix: handle empty repositories",
    "files_changed": 1,
    "insertions": 10,
    "deletions": 0,
    "changes": [ { "path": "src/main.rs", "kind": "modified" } ]
  }
]
```

O schema é versionado e documentado campo a campo em **[docs/JSON.md](docs/JSON.md)**, com a política de estabilidade e receitas de `jq`.

## Como os números são definidos

Numa ferramenta de estatísticas as definições precisas importam, então aqui estão:

- **Histórico analisado.** Os commits alcançáveis a partir do `HEAD` ou de `--branch`. Commits que só existem em branches não mescladas ficam de fora. Merges contam como commits.
- **Datas.** Cada commit é posicionado pela **data do autor, em UTC**. Dias da semana, horas e meses são faixas em UTC. `--since` começa às 00:00:00 do dia e `--until` vai até as 23:59:59 do dia.
- **Contribuidores.** Um autor é identificado pelo **endereço de e-mail** (sem diferenciar maiúsculas), então `Bob` e `Bob Smith` com o mesmo e-mail são um só contribuidor. O nome exibido é o do commit mais recente. O `.mailmap` não é aplicado.
- **Percentuais.** A parcela de um contribuidor nos commits que passaram nos filtros (o terminal mostra uma casa decimal; o JSON, duas).
- **Arquivo modificado.** Um arquivo é *modificado* uma vez por commit em que aparece alterado em relação ao primeiro pai: criado, modificado, apagado ou destino de um rename. Um arquivo tocado em dois commits tem duas modificações. Renames são detectados (50 % de similaridade) e contados no caminho novo; o histórico não é seguido através de renames, e cópias não são detectadas.
- **Inserções e deleções.** Linhas adicionadas e removidas por commit em relação ao primeiro pai. O primeiro commit é comparado com uma árvore vazia. **Merges não contribuem com arquivos nem linhas**, porque as mudanças deles já estão nos commits mesclados. Arquivos binários contam como arquivo alterado com zero linhas.
- **Arquivos, extensões e linguagens** descrevem um **retrato**: a árvore de arquivos na ponta do histórico analisado (`HEAD` ou `--branch`). Ignoram `--since`, `--until` e `--author`. Os *arquivos mais modificados* e todo o resto vêm do histórico filtrado.
- **Extensões.** A última extensão, em minúsculas (`archive.tar.gz` → `.gz`). Arquivos como `Makefile` e dotfiles como `.gitignore` não têm extensão e aparecem como `(none)`.
- **Linguagens.** Derivadas da extensão pela tabela em [`src/analysis/languages.rs`](src/analysis/languages.rs). Os percentuais são por **número de arquivos** e só consideram arquivos com linguagem reconhecida: textos e dados (Markdown, JSON, TOML…) não diluem o gráfico.
- **Branches.** Branches locais (numa URL remota: as branches do remoto). **Tags** incluem as leves e as anotadas.
- **Resultados vazios** não são erros: um repositório sem commits, ou um filtro de autor/data que não encontra nada, imprime uma mensagem (ou um JSON válido com `"total": 0`) e sai com `0`.

## Desempenho

O `gitscope` processa o histórico de forma **incremental**: uma passada pelos commits, um diff por vez, sem guardar nada por commit, a menos que `--all` seja usado. Os filtros de autor e data são aplicados *antes* do diff, então execuções com `--author` e `--since` pulam a parte cara em todo commit que excluem.

Medido com um build release num repositório de 361 commits (Windows, 12 núcleos):

| Execução | Tempo |
|---|---|
| Só o percurso (o filtro não casa com nenhum commit) | 0,06 s |
| Relatório completo | 3,9 s |
| `git log --numstat`, para comparação | 1,0 s |

Praticamente todo o tempo vai para a contagem de linhas na libgit2. Este repositório é quase um pior caso: tem dois arquivos de ~1 MB editados em quase 300 commits cada um, então centenas de megabytes de texto precisam ser comparados. A detecção de renames custa quase nada, e os resultados batem com o `git` (número de commits, contagem por autor, inserções, deleções, número de arquivos e arquivos mais modificados foram conferidos). Diffs em paralelo são o primeiro candidato se um dia isso precisar ficar mais rápido; veja o [roadmap](#roadmap).

### Benchmarks

```bash
cargo bench                                # um repositório com 1000 commits, 7 execuções de cada
GITSCOPE_BENCH_COMMITS=5000 cargo bench    # um maior
```

O benchmark monta um repositório sintético (20 arquivos, 4 autores) e mede cada etapa. Melhores tempos numa máquina Windows com 12 núcleos, para 1000 commits:

| Cenário | Melhor tempo |
|---|---|
| Só o percurso (nenhum commit casa, sem diffs) | 95 ms |
| Relatório completo | 2,2 s |
| Relatório completo com detalhe por commit (`--all`) | 2,2 s |
| Um autor (diff só dos commits dele) | 0,75 s |
| Renderizar o relatório do terminal | 0,4 ms |
| Renderizar o JSON (com detalhe por commit) | 0,4 ms |
| Renderizar um template Markdown | < 0,01 ms |

Os números dependem da máquina, então use-os para comparar versões entre si. O repositório sintético tem objetos soltos (sem pacote), o que faz cada diff ler mais arquivos do que num repositório empacotado. O que vale em geral é o formato: percorrer é barato, os diffs são o custo, o `--all` quase não acrescenta nada e renderizar é de graça.

## Como funciona

```
Git repository ─► repository.rs ─► CommitRecord ─► analysis/* ─► RepositoryStats ─┬─► terminal
                  (só git2)        (modelo)        (puro)                          └─► JSON
```

As camadas são estritamente separadas: só o `repository.rs` fala com o `git2`, o `analysis/` nunca imprime, e o `output/` (terminal, JSON e Markdown) nunca recalcula. Veja **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** para os detalhes e para saber como estender a ferramenta.

O crate é ao mesmo tempo um binário (`gitscope`) e uma biblioteca (`gitscope::analyze_path`), que é o que os testes de integração usam.

## Desenvolvimento

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

Os testes de integração montam repositórios pequenos e determinísticos com o `git2` em diretórios temporários (autores e datas fixos), então não precisam do executável `git` nem de rede. A análise remota é testada clonando URLs `file://` e falhando contra uma porta local fechada. Eles cobrem repositórios vazios, caminhos inválidos, vários autores, meses e horas diferentes, todo tipo de mudança de arquivo, merges, filtros, JSON e `--all`.

O que permanece estável entre versões está descrito em **[docs/STABILITY.md](docs/STABILITY.md)**; como uma release é feita, em **[docs/RELEASING.md](docs/RELEASING.md)**.

## Roadmap

- [x] **v0.1 — Base:** abrir um repositório, commits, branches, tags, contribuidores, primeiro/último commit, modelo de dados
- [x] **v0.2 — Estatísticas:** commits por autor / mês / dia da semana / hora, arquivos, extensões, linguagens, atividade
- [x] **v0.3 — CLI:** `--since`, `--until`, `--author`, `--branch`, `--verbose`, saída formatada
- [x] **v0.4 — Exportação:** `--json` com um schema estável e documentado
- [x] **v0.5 — Inspeção de commits:** `--all`, histórico por autor, arquivos, inserções e deleções por commit
- [x] **v0.6 — Repositórios remotos:** analisar uma URL por um clone temporário que é limpo depois
- [x] **v0.7 — Templates Markdown:** `{variáveis}` no seu próprio README, `gitscope update` / `markdown` / `variables`, blocos, gravação atômica e segura
- [x] **v0.8 — TUI:** `gitscope tui`, uma interface interativa com o [`ratatui`](https://ratatui.rs): seis abas, lista de commits filtrada enquanto você digita, detalhes de cada commit, escolha do idioma (inglês / português)
- [x] **v1.0 — Estável:** uma política de estabilidade, benchmarks básicos (`cargo bench`), um workflow de release que gera binários para quatro plataformas (ainda a ser executado pela primeira vez) e a documentação de instalação

Ideias em avaliação (sem compromisso, sem ordem específica):

- Mais variáveis compostas: `{commit:first}`, `{commit:last}`, `{language:Rust:files}`, mais campos de `{author:NAME:FIELD}` (`files_changed`, `percentage`, primeiro/último commit).
- `gitscope update --check`, que falha quando o README está desatualizado (para CI), e uma GitHub Action que o mantém em dia.
- Suporte a `.mailmap`, para uma pessoa com vários e-mails contar uma vez só.
- Diffs em paralelo, para acelerar históricos grandes (veja [Desempenho](#desempenho)).

## Contribuindo

Contribuições são bem-vindas. Leia primeiro o **[CONTRIBUTING.md](CONTRIBUTING.md)**: ele explica o escopo do projeto (uma CLI local, deliberadamente pequena), como rodar as verificações e como adicionar uma linguagem.

## Licença

Licenciado, à sua escolha, sob

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- Licença MIT ([LICENSE-MIT](LICENSE-MIT))

Veja [LICENSE](LICENSE).
