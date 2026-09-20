# Política de estabilidade

**Português** · [English](STABILITY-EN.md)

A partir da versão 1.0, o GitScope segue o [Versionamento Semântico](https://semver.org/lang/pt-BR/) para o que esta página lista como estável. Ela diz exatamente o que isso cobre, para você saber sobre o que pode construir scripts, jobs de CI e READMEs.

## O que o número da versão garante

| Área | Garantia |
|---|---|
| Comandos, opções e o que fazem | Estável. Comandos e opções novos podem surgir em versões menores; remover ou renomear um, ou mudar o que ele significa, é uma mudança maior. |
| Códigos de saída (`0`, `1`, `2`) e a separação stdout/stderr | Estável. |
| Saída JSON | Estável, versionada por `schema_version` (a política está em [JSON.md](JSON.md)). |
| Templates Markdown: marcadores, sintaxe `{variável}`, escape e as variáveis documentadas | Estável. Variáveis novas podem surgir em versões menores; remover uma ou mudar o significado do valor dela é uma mudança maior. |
| Arquivo de configuração | Estável: a chave `language`. Chaves novas podem surgir; chaves desconhecidas são ignoradas. |

## O que ele não garante

| Área | Por quê |
|---|---|
| O texto do relatório no terminal | É para pessoas: rótulos, colunas e barras podem mudar em qualquer versão. Para scripts, use `--json`. |
| A interface interativa (`gitscope tui`) | Layout, teclas e textos podem mudar em versões menores; as teclas documentadas são mantidas sempre que possível. |
| O texto das mensagens de erro | Só vale que vão para o stderr com código de saída 1. |
| Os detalhes internos da biblioteca Rust | Veja abaixo. |

## A biblioteca Rust

O crate é, ao mesmo tempo, um binário e uma biblioteca. Estes pontos de entrada e tipos seguem o Versionamento Semântico a partir da 1.0: `analyze_path`, `analyze_repository`, `open_source`, `is_remote`, `analysis::Options`, os tipos de `model` (os campos deles espelham o schema JSON), `GitScopeError` (com `non_exhaustive`, então novas variantes não quebram quem a usa) e `config::{Config, Language}`.

Todo o resto que é `pub` (os acumuladores da análise, os renderizadores, a interface, `repository`, `remote` e os detalhes de `output::markdown` e `output::variables`) existe para os testes e os benchmarks e pode mudar em qualquer versão. Não dependa disso.

## Mudando algo que é estável

Uma mudança incompatível sobe a versão maior e aparece no [CHANGELOG](../CHANGELOG.md), em *Alterado* ou *Removido*. Quando possível, o comportamento antigo é mantido por uma versão menor com um aviso no stderr antes de ser removido.
