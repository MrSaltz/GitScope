# Contribuindo com o gitscope

**Português** · [English](CONTRIBUTING-EN.md)

Obrigado pelo interesse! O GitScope é pequeno de propósito: **uma CLI local que analisa repositórios Git.** Por favor, mantenha as contribuições dentro desse escopo.

## Escopo

Dentro do escopo: estatísticas melhores ou novas, correções de precisão, acabamento da saída, trabalho de desempenho apoiado em medições, documentação, testes e os itens do [roadmap](README.md#roadmap).

Fora do escopo (por favor, não abra PRs sobre isso): servidores web ou dashboards, bancos de dados, autenticação ou contas (isso inclui SSH e pedidos de credenciais em clones remotos), sincronização na nuvem, telemetria, recursos de IA ou qualquer coisa que colete dados externos. Na dúvida, abra uma issue antes.

## Primeiros passos

Requisitos: Rust 1.87+ e um compilador C (veja o [README](README.md#instalação)). O executável `git` **não** é necessário, nem para rodar os testes, e os testes nunca usam a rede. No Linux você também precisa do `pkg-config` e dos arquivos de desenvolvimento do OpenSSL (ou use `--no-default-features`, veja o README).

```bash
git clone https://github.com/MrSaltz/GitScope.git
cd gitscope
cargo test
```

Antes de enviar uma mudança, confira que tudo isto passa (o CI roda as mesmas verificações):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --no-default-features   # o build só local precisa continuar funcionando
```

## Regras do projeto

Leia [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Em resumo:

- Só o `src/repository.rs` usa o `git2`.
- O `src/analysis/` é puro: sem imprimir, sem acessar o Git.
- O `src/output/` só formata; nunca recalcula estatísticas.
- Nunca chame o `git` por shell.
- Nunca imprima uma URL sem passar por `remote::redact`: ela pode conter credenciais.
- Processe o histórico de forma incremental; não guarde diffs nem dados por commit na memória, a menos que um recurso precise.
- Prefira a simplicidade: nada de dependência, abstração ou flag nova sem necessidade clara.

## Comentários no código

Comente pouco: só o que o código não deixa claro, como uma regra não óbvia, uma armadilha ou o *porquê* de uma escolha, nunca o *quê* que ele já diz. Cada comentário tem no máximo uma ou duas linhas e é escrito em **português**. A exceção são os comentários `///` de `src/cli.rs`: o clap os transforma no texto do `--help`, então fazem parte da interface e ficam em inglês, como toda outra mensagem que a ferramenta imprime.

## Documentação em dois idiomas

O **português é o idioma principal**; cada arquivo `.md` tem um par em inglês com o sufixo `-EN`:

| Português (principal) | Inglês |
|---|---|
| `README.md` | `README-EN.md` |
| `CONTRIBUTING.md` | `CONTRIBUTING-EN.md` |
| `CHANGELOG.md` | `CHANGELOG-EN.md` |
| `docs/ARCHITECTURE.md` | `docs/ARCHITECTURE-EN.md` |
| `docs/JSON.md` | `docs/JSON-EN.md` |
| `docs/MARKDOWN.md` | `docs/MARKDOWN-EN.md` |
| `docs/STABILITY.md` | `docs/STABILITY-EN.md` |
| `docs/RELEASING.md` | `docs/RELEASING-EN.md` |

Ao mudar um, **atualize o outro no mesmo PR**. O teste `tests/docs.rs` ajuda: ele falha se algum link ou âncora estiver quebrado e se os dois idiomas de um documento tiverem estruturas diferentes (número de títulos, blocos de código, tabelas e links). Ele não confere a tradução em si; isso é com você. Os links de cada idioma apontam para os documentos do mesmo idioma. Os exemplos de saída do programa ficam iguais nos dois, porque a ferramenta imprime em inglês.

## Testes

Toda mudança de comportamento precisa de um teste.

- **Testes unitários** ficam ao lado do código. Para estatísticas, monte `CommitRecord`s com `analysis::testutil`.
- **Testes de integração** em `tests/` montam repositórios reais com o auxiliar `TestRepo`, em `tests/common/mod.rs`:

  ```rust
  let t = TestRepo::new();
  t.by("Alice", "alice@example.com")
      .at("2026-01-03 12:30")
      .message("feat: add statistics")
      .write("src/main.rs", "fn main() {}\n")
      .commit();
  ```

  Use datas fixas para os resultados serem determinísticos.
- **O schema JSON é travado** por `json_schema_is_stable`, em `tests/cli.rs`. Se você o mudar de propósito, atualize [docs/JSON.md](docs/JSON.md) e o changelog, e leia antes a política de estabilidade que está lá.

## Adicionando uma variável Markdown

Acrescente uma entrada em `standard_variables()`, em `src/output/variables.rs` (nome, grupo, descrição, resolvedor), um teste junto dos outros nesse arquivo e uma linha em [docs/MARKDOWN.md](docs/MARKDOWN.md) (e em `docs/MARKDOWN-EN.md`): um teste falha quando uma variável registrada não está documentada. Os resolvedores só leem o `RepositoryStats`; se um valor precisa de dados novos, calcule-os em `analysis/` e documente em [docs/JSON.md](docs/JSON.md).

## Adicionando uma linguagem

Acrescente uma linha `(extensão, linguagem)` em `LANGUAGES`, em `src/analysis/languages.rs`. As extensões são minúsculas, sem o ponto, e devem ser únicas.

## Commits e pull requests

- Mantenha os pull requests focados; um assunto por PR.
- Use mensagens de commit claras e no imperativo. [Conventional Commits](https://www.conventionalcommits.org) (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`) são bem-vindos, mas não obrigatórios.
- Atualize `README.md` / `docs/` e `CHANGELOG.md` (e as versões `-EN`) quando o comportamento mudar.
- Afirmações de desempenho precisam de números: descreva o repositório, o comando e os tempos de antes e depois.

## Reportando bugs

Inclua o `gitscope --version`, o seu sistema operacional, o comando exato e, se possível, um repositório mínimo (ou a sequência de commits) que reproduza o problema. A saída de `--verbose` ajuda.

## Licença

Ao contribuir, você concorda que o seu trabalho é licenciado sob **MIT OR Apache-2.0**, como o projeto (veja [LICENSE](LICENSE)).
