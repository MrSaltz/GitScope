# Changelog

**Português** · [English](CHANGELOG-EN.md)

Todas as mudanças relevantes deste projeto estão documentadas aqui.
O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o projeto usa [Versionamento Semântico](https://semver.org/lang/pt-BR/) (antes da 1.0, versões menores podem mudar a CLI).

A saída JSON tem versão própria, `schema_version`; veja [docs/JSON.md](docs/JSON.md).

## [1.0.0] - 2026-09-20

A primeira versão estável: a linha de comando, os códigos de saída, o schema JSON, a sintaxe dos templates Markdown e o arquivo de configuração passam a seguir o Versionamento Semântico (veja [docs/STABILITY.md](docs/STABILITY.md)).

### Adicionado

- **Política de estabilidade** ([docs/STABILITY.md](docs/STABILITY.md)): o que o número da versão garante, o que não garante e como mudanças incompatíveis são tratadas.
- **Benchmarks** (`cargo bench`, `benches/analysis.rs`): um repositório sintético, com a medição do percurso, da análise completa, de um autor e de cada renderizador. Sem framework e sem dependência nova.
- **Workflow de release** (`.github/workflows/release.yml`): ao receber uma tag `vX.Y.Z`, confere a versão, compila binários para Linux x86_64, Windows x86_64 e macOS (arm64 e x86_64) e cria a Release no GitHub com os arquivos e o `SHA256SUMS` (tags com hífen viram pré-releases). Ainda não foi executado; a primeira tag é o teste dele (veja [docs/RELEASING.md](docs/RELEASING.md)).
- Feature Cargo `vendored-openssl`, que embute o OpenSSL (usada no binário de release do Linux).
- Documentação de instalação dos binários prontos e do processo de release ([docs/RELEASING.md](docs/RELEASING.md)).

### Alterado

- `GitScopeError`, `MarkdownError`, `ProblemKind` e `ChangeKind` são `non_exhaustive`, então acrescentar uma variante depois não é uma mudança incompatível.
- O crate é empacotado sem a pasta `.github/`; o `cargo publish --dry-run` passa (55 arquivos). `repository` e `homepage` ainda precisam ser preenchidos.

## [0.8.0] - 2026-09-20

### Adicionado

- **Interface interativa.** `gitscope tui [PATH_OR_URL]`, feita com o `ratatui`: abas de visão geral, contribuidores, arquivos, atividade e commits; listas selecionáveis; uma lista de commits filtrada enquanto você digita (autor, e-mail, mensagem ou hash) que mostra os arquivos, as inserções e as deleções do commit selecionado; `Enter` num contribuidor abre os commits dele; uma janela de ajuda (`?`). Aceita os filtros de sempre e URLs.
- **Aba Configurations** (tecla `6`) para mudar o idioma da interface, **English** ou **Português (Brasil)**: a tela é traduzida na hora e a escolha é guardada num pequeno arquivo JSON no diretório de configuração do usuário (`%APPDATA%\gitscope\config.json` no Windows, `~/Library/Application Support/gitscope/config.json` no macOS, `$XDG_CONFIG_HOME/gitscope/config.json` ou `~/.config/gitscope/config.json` no Linux; `GITSCOPE_CONFIG` o substitui). Um arquivo ilegível é ignorado com um aviso. Os textos ficam numa struct completa por idioma, então uma tradução faltando é erro de compilação.
- Mensagens, autores e caminhos de arquivos longos são quebrados em linhas dentro do detalhe do commit, em vez de saírem da tela (por palavras, medidos em colunas da tela para acentos e caracteres largos contarem certo; as linhas seguintes ficam alinhadas sob o valor).
- Testes da interface: cada tecla sobre o estado (sem precisar de terminal) e cada tela desenhada no `TestBackend` em memória do ratatui, inclusive em tamanhos minúsculos e com repositórios vazios.

### Alterado

- As funções de barra, sparkline e truncamento saíram do renderizador de terminal para o módulo compartilhado `output::format` (a saída não muda).

### Limitações conhecidas

- Precisa de um terminal de verdade e se recusa a rodar em pipes. A análise roda antes de a interface abrir (sem tela de progresso) e o detalhe de todos os commits fica na memória.
- Só a interface é traduzida; o relatório, as mensagens e os outros comandos continuam em inglês.
- Sem suporte a mouse. Os testes automatizados nunca usam um terminal real: conferem o estado e o desenho, não a preparação e a restauração do terminal.

## [0.7.0] - 2026-09-20

### Adicionado

- **Templates Markdown.** Escreva `{variáveis}` no seu próprio README e o GitScope as preenche, sem mudar mais nada:
  - `gitscope update [FILE]` atualiza no lugar os blocos `<!-- gitscope:start -->` / `<!-- gitscope:end -->` de um arquivo (padrão: `README.md` na raiz do repositório). Cada bloco guarda seu template num comentário HTML invisível para poder ser atualizado de novo; repetir uma atualização com os mesmos dados não muda nada.
  - `gitscope markdown FILE [-o OUT]` imprime um template com as variáveis resolvidas e nunca o modifica; um arquivo sem blocos é resolvido por inteiro.
  - `gitscope variables` lista as variáveis.
- Variáveis: `{repository_name}`, `{first_commit}`, `{last_commit}`, `{period}`, `{branches}`, `{tags}`, `{commits}`, `{first_commit_message}`, `{last_commit_message}`, `{most_active_hour}`, `{contributors}`, `{top_author}`, `{top_author_commits}`, `{top_author_percentage}`, `{files}`, `{insertions}`, `{deletions}`, `{language_count}`, `{top_language}` e a composta `{author:NAME:commits|insertions|deletions}`. Ficam num registro central: acrescentar uma é uma única entrada.
- Segurança: o arquivo é validado (blocos, variáveis) e renderizado em memória antes de qualquer gravação; em qualquer erro ele fica intocado; as gravações são atômicas e mantêm as permissões; arquivos sem mudança não são regravados; quebras de linha, BOM e a quebra final são preservados. Variáveis desconhecidas falham com a lista das disponíveis e um "você quis dizer"; `--keep-unknown` as transforma em avisos.
- JSON: `commits` ganha `first_message`, `last_message`, `insertions` e `deletions` (aditivo; `schema_version` continua 1).
- Documentação: [docs/MARKDOWN.md](docs/MARKDOWN.md). Toda a documentação passa a existir em **português (principal)** e em **inglês** (arquivos `-EN`), com um teste (`tests/docs.rs`) que confere links e estrutura dos dois idiomas.

### Alterado

- **O projeto agora se chama GitScope** (antes era GitStats e, por pouco tempo, GitStatus, nome que já existe no crates.io). O crate, a biblioteca e o binário são `gitscope`, o cabeçalho do terminal diz `GITSCOPE` e o tipo de erro é `GitScopeError`. Os marcadores de bloco são `gitscope:start` / `gitscope:end`; os antigos `gitstatus:` e `gitstats:`, e o comentário de template que uma versão anterior escreveu, continuam aceitos e são migrados no próximo `update`.
- As opções do relatório não podem ser combinadas com `update`, `markdown` ou `variables`. Um nome de subcomando só é especial como primeiro argumento, então use `./update` para um diretório com esse nome.

### Limitações conhecidas

- Um bloco cujo template contém `-->` não pode ser guardado pelo `update` (isso encerraria o comentário HTML que o guarda); use `markdown -o` com um arquivo de template separado.
- Os valores entram como são: um `|` numa mensagem de commit quebra a célula de tabela onde ela for colocada.

## [0.6.0] - 2026-09-20

### Adicionado

- **Repositórios remotos.** `gitscope <URL>` clona o repositório num diretório temporário, analisa e apaga o clone. Suportadas: `https://`, `http://`, `git://`, `file://`. Todas as opções e filtros funcionam em repositórios remotos.
- O clone é bare (sem árvore de trabalho), completo e limpo tanto em caso de sucesso quanto de falha. O progresso aparece no stderr quando ele é um terminal.
- Em clones, *branches* conta as branches do remoto e `--branch` aceita branches remotas pelo nome simples ou como `origin/<nome>`.
- Credenciais embutidas em URLs são ocultadas de toda mensagem.
- A feature Cargo `remote` (padrão) liga o HTTPS via `git2/https`; `--no-default-features` gera um binário só local, sem TLS/OpenSSL.
- Biblioteca: `open_source`, `analyze_repository`, `is_remote`, `Repository::clone_to_temp`, `Repository::close`, módulo `remote`.

### Alterado

- Mensagens de erro de clone mais limpas, com uma dica quando o host responde 401/403 (repositório privado ou inexistente).
- `tempfile` agora é uma dependência normal.

### Limitações conhecidas

- Só transportes sem autenticação: SSH e repositórios privados não são suportados.
- Se o processo for morto (por exemplo, com Ctrl-C) durante ou depois de um clone, o diretório temporário `gitscope-*` pode ficar para trás.

## [0.5.0] - 2026-09-20

Primeira versão pública: o MVP completo (roadmap v0.1 a v0.5).

### Adicionado

- Análise de repositórios locais pela libgit2 (`git2`), a partir de um diretório de trabalho, de um diretório `.git` ou de um repositório bare. O executável `git` não é necessário.
- Fatos do repositório: nome, branches locais, tags.
- Estatísticas de commits: total, primeiro e último commit, commits por dia da semana e por hora, hora mais ativa.
- Contribuidores: commits, percentual, arquivos alterados, inserções, deleções, primeiro e último commit. A identidade é o e-mail em minúsculas.
- Estatísticas de arquivos: arquivos no retrato, extensões, arquivos mais modificados. Detecção de arquivos criados, modificados, apagados e renomeados.
- Distribuição de linguagens a partir de um mapeamento de extensões guiado por tabela.
- Atividade por mês, com os meses calmos preenchidos com zero.
- Filtros: `--since`, `--until`, `--author` (nome ou e-mail, sem diferenciar maiúsculas), `--branch`. Os filtros se combinam.
- Visão do autor (`--author`) com um resumo por contribuidor.
- `--all`: todos os commits que casam com os filtros, com autor, data, mensagem, arquivos alterados, inserções e deleções. `--verbose` imprime o bloco completo de cada commit.
- Diagnósticos de `--verbose` no stderr; `--top` para limitar as listas ordenadas na saída do terminal.
- Exportação `--json` com schema versionado e documentado (`schema_version` 1).
- Saída formatada no terminal, com seções, tabelas alinhadas, barras e um sparkline.
- Documentação (README, schema JSON, arquitetura, contribuição), licença dupla MIT/Apache-2.0 e um workflow do GitHub Actions (formatação, clippy, testes em Linux, Windows e macOS, MSRV, build release).

### Limitações conhecidas

- As estatísticas de linhas são calculadas com a libgit2, que é mais lenta que o `git` em arquivos muito grandes (veja *Desempenho* no README).
- O `.mailmap` não é aplicado; a mesma pessoa com dois e-mails conta como dois contribuidores.
- Merges são contados, mas não contribuem com arquivos nem linhas.
- Cópias não são detectadas e o histórico não é seguido através de renames.
- As branches contadas são só as locais.
