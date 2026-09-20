# Arquitetura

**Português** · [English](ARCHITECTURE-EN.md)

O GitScope é um pipeline pequeno. Cada etapa tem uma função e sabe o mínimo possível sobre as outras.

```
Repositório Git
      │
      ▼
repository.rs      Coleta de dados (abrir, clonar, percorrer, diff). Único módulo
      │            que importa o git2. Entrega CommitRecord um de cada vez.
      │            (remote.rs só interpreta URLs: decide entre local e clone.)
      ▼
model.rs           Tipos de domínio (CommitRecord, RepositoryStats, ...). Sem git2.
      │
      ▼
analysis/          Estatísticas puras sobre CommitRecord. Nunca imprime, nunca toca no Git.
      │
      ├──────────────┐
      ▼              ▼
output/terminal.rs   output/json.rs   output/markdown.rs (+ variables.rs)   output/tui/
                                      Só formatam. Nunca recalculam nada.
```

O `cli.rs` define os argumentos (clap), o `error.rs` os erros de domínio (thiserror), e o `main.rs` junta tudo e transforma erros em códigos de saída (anyhow).

## Regras que mantêm o projeto sustentável

1. **O `git2` fica no `repository.rs`.** Todo o resto trabalha com tipos do `model`. É isso que permite testar o `analysis/` com dados escritos à mão e trocar o Git por outro backend (ou um mock) no futuro.
2. **O `analysis/` é puro.** Sem E/S, sem imprimir.
3. **O `output/` só formata.** Se um renderizador precisa de um número, a análise deve produzi-lo; o schema JSON é literalmente o [`RepositoryStats`](../src/model.rs) serializado.
4. **Sem o executável `git`.** Nunca chame o `git` por shell. Use o `git2` (libgit2).
5. **Nada fica para trás, nada vaza.** Um clone temporário pertence ao handle `Repository` e é apagado quando ele é fechado ou liberado. Credenciais em URLs são ocultadas (`remote::redact`) antes de qualquer mensagem ser montada.
6. **Incremental.** Os commits passam pelo pipeline um de cada vez. Um `Diff` nunca sobrevive a uma iteração do laço, e nada é retido por commit, a menos que `--all` peça.

## Fluxo de dados em detalhe

1. `open_source` (em `lib.rs`) abre um caminho local ou, para uma URL que não existe no disco, clona (bare, num diretório temporário) com `Repository::clone_to_temp`. `analyze_repository` valida os [`Filters`](../src/model.rs) e lê os fatos do repositório (nome, branches, tags). `analyze_path` é o atalho só para repositórios locais.
2. `Repository::commits` percorre o histórico a partir da ponta (`HEAD` ou `--branch`) com um `revwalk`. Para cada commit, lê autor e data e aplica **primeiro os filtros de autor e data**, antes de qualquer diff. Os commits que sobram são comparados com o primeiro pai (com detecção de renames) e viram um `CommitRecord`. Merges não passam por diff.
3. `analysis::analyze` entrega cada `CommitRecord` a um conjunto de *acumuladores*, um por assunto:

   | Módulo | Acumula |
   |---|---|
   | `commits.rs` | totais, primeiro/último, faixas por dia da semana e hora |
   | `contributors.rs` | commits, arquivos, linhas e datas por identidade |
   | `files.rs` | contagem de modificações por caminho; contagem de extensões do retrato |
   | `activity.rs` | commits por mês |
   | `languages.rs` | parcela de cada linguagem no retrato (guiado por tabela) |

   Com `Options { commit_details: true }` (`--all`), guarda também um `CommitInfo` por commit.
4. `Repository::snapshot_files` lista a árvore de arquivos na ponta; alimenta as estatísticas baseadas no retrato.
5. O resultado, `RepositoryStats`, vai para `output::terminal::render` ou `output::json::render`.
6. Para Markdown, o `output::markdown` divide o documento em linhas, encontra e valida os blocos `gitscope`, resolve cada `{variável}` pelo `VariableRegistry` (`output/variables.rs`, que só *lê* o `RepositoryStats`) e remonta o documento sem alterar nada além das variáveis. Depois o `update_file` o grava de forma atômica.
7. Para a interface, o `main.rs` analisa uma vez (com `commit_details`) e entrega o resultado a `output::tui::run`. O `tui/app.rs` é o estado (aba, seleções, filtro de commits) e o tratamento do teclado, um valor comum sem desenho; o `tui/ui.rs` o desenha com o `ratatui`; o `tui/mod.rs` cuida do terminal e do laço de eventos. O `tui/i18n.rs` guarda os textos da interface, uma struct `Texts` completa por idioma, e o `config.rs` lê e grava o pequeno arquivo JSON onde as escolhas (o idioma) ficam.

## Estendendo o GitScope

### Adicionar uma linguagem

Acrescente uma linha em `LANGUAGES`, em [`src/analysis/languages.rs`](../src/analysis/languages.rs):

```rust
("zig", "Zig"),
```

As extensões são minúsculas e únicas (um teste unitário garante isso). O código da análise não muda.

### Adicionar uma estatística

1. Acrescente o tipo do resultado em `model.rs` (com `Serialize`) e um campo em `RepositoryStats`.
2. Escreva um acumulador em `analysis/` com `add(&CommitRecord)` e `finish()`, mais testes unitários com `analysis::testutil`.
3. Chame-o em `analysis::analyze`.
4. Renderize em `output/terminal.rs`. O JSON incorpora automaticamente.
5. Isso muda o schema JSON público: atualize o [`JSON.md`](JSON.md), as asserções de chaves em `tests/cli.rs` e o changelog.

### Adicionar uma variável Markdown

Acrescente uma entrada em `standard_variables()`, em [`src/output/variables.rs`](../src/output/variables.rs):

```rust
plain("Commits", "commits", "Number of commits", |c, _| {
    Ok(thousands(c.stats.commits.total))
}),
```

Um nome, um grupo, uma descrição e um resolvedor. O resolvedor recebe os argumentos escritos depois do nome (`{author:Alice:commits}` -> `["Alice", "commits"]`), então variáveis compostas não precisam de nada extra; declare-as com `VariableDef { arguments: "NAME:FIELD", .. }`. O `gitscope variables`, as mensagens de erro, as sugestões de digitação e os testes que conferem o registro passam a incluí-la automaticamente. Atualize o [`MARKDOWN.md`](MARKDOWN.md): um teste falha se uma variável não estiver nele. Se o valor precisa de dados que o modelo não tem, acrescente-os à análise (e à documentação do JSON), nunca ao resolvedor.

### Adicionar um idioma da interface

Acrescente uma variante a `Language`, em `src/config.rs` (o nome no `serde`, o nome na própria língua e o lugar em `ALL`), depois uma `Texts` estática completa para ele em `src/output/tui/i18n.rs` e uma linha em `texts()`. O compilador aponta tudo que falta, e os testes conferem os modelos `{...}` e que nenhum texto está vazio.

### Precisa de mais dados do Git

Estenda só o `CommitRecord` e o `repository.rs`. Mantenha tipos do `git2` fora do modelo.

## Como testamos

| Camada | Como é testada |
|---|---|
| `analysis/*`, funções auxiliares de `output/terminal.rs`, `cli.rs` | Testes unitários ao lado do código, com valores montados à mão. |
| `repository.rs` + `analysis` | `tests/repository.rs` e `tests/analysis.rs`: repositórios reais montados com o `git2` por `tests/common::TestRepo` (autores e datas fixos, sem o executável `git`). |
| Ponta a ponta | `tests/cli.rs` roda o binário compilado: códigos de saída, mensagens, saída do terminal, schema JSON. |
| Motor de Markdown | `output/markdown/tests.rs` e `output/variables.rs`: estatísticas montadas à mão; blocos, escapes, CRLF/BOM, idempotência, gravações atômicas, casos de erro. |
| Comandos de Markdown | `tests/markdown.rs` roda `update`, `markdown` e `variables` em repositórios reais e confere cada variável contra o relatório JSON. |
| Interface | `output/tui/tests.rs`: cada tecla sobre o `App` (sem terminal) e cada tela desenhada no `TestBackend` em memória do ratatui, inclusive em tamanhos minúsculos e com repositórios vazios. O `tests/cli.rs` confere que ela se recusa a rodar sem terminal. |
| Benchmarks | `benches/analysis.rs` (`cargo bench`): monta um repositório sintético com o `TestRepo` e mede o percurso, a análise completa, a análise de um autor e cada renderizador. Não usa framework; imprime o melhor tempo e a mediana. |
| Remoto | `tests/remote.rs` clona URLs `file://` de `TestRepo`s (sem rede), aponta o diretório temporário do binário para uma pasta de teste para provar a limpeza e falha contra uma porta local fechada. |

## Decisões de projeto que vale conhecer

- **Data do autor, em UTC.** Determinística entre máquinas e fusos, e é a data em que o contribuidor escreveu a mudança.
- **O e-mail é a identidade.** Nomes variam; e-mails raramente. Suporte a `.mailmap` é uma possível extensão.
- **Retrato vs. histórico.** "Quantos arquivos / quais linguagens" descreve o projeto como ele é; "quem fez o quê e quando" descreve o histórico filtrado. Misturá-los (por exemplo, filtrar a contagem de arquivos por autor) não faria sentido.
- **Clone bare.** Na análise remota só o histórico importa, então o clone não tem árvore de trabalho: é mais rápido, usa menos disco e nenhum hook ou filtro de checkout pode rodar.
- **Feature Cargo `remote`.** O suporte a HTTPS traz TLS (OpenSSL no Linux). Vem ligado por padrão e pode ser desligado (`--no-default-features`); tudo o que é local continua funcionando.
- **O `update` guarda o template.** Trocar `{commits}` por `361` destrói o template, e um README que não pode ser renovado tem pouca utilidade. Por isso um bloco atualizado guarda seu template num comentário HTML (invisível ao renderizar), como os geradores de código que mantêm o fonte ao lado da saída gerada. O `markdown` é o par puro: nunca guarda nada e nunca grava a entrada.
- **Só as variáveis mudam.** O motor trabalha sobre o texto original e encaixa os resultados; nunca reformata, então CRLF, BOM, tabelas e HTML sobrevivem. Os valores são neutralizados (`<!--`, `-->`) e nunca reprocessados, então dados não conseguem alterar a estrutura do documento.
- **Analisa primeiro, navega depois.** A interface analisa uma vez antes de abrir (sem threads, sem tela de carregamento) e depois só navega; filtrar a lista de commits esconde itens que já estão carregados. O estado não tem código de desenho, então as teclas são testadas sem terminal.
- **O idioma é uma configuração da interface.** Os textos são uma struct por idioma (uma tradução faltando não compila) e a escolha fica num pequeno arquivo JSON no diretório de configuração do usuário, gravado de forma atômica como todo outro arquivo que o GitScope grava. O relatório, os erros e os outros comandos continuam em inglês.
- **`HEAD` por padrão.** Coerente com o retrato dos arquivos e com o `git log`; use `--branch` para outra linha de histórico.
- **Desempenho é medido, não presumido.** Veja a seção *Desempenho* do README, com as medições e o gargalo identificado.
