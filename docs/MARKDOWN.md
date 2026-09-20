# Templates Markdown

**Português** · [English](MARKDOWN-EN.md)

O GitScope pode preencher o seu README com estatísticas. Você escreve o Markdown exatamente como quiser e coloca **variáveis** como `{commits}` onde os números devem ficar. O GitScope troca **só as variáveis**; todo o resto fica como você escreveu.

```markdown
🚀 Este projeto tem {commits} commits de {contributors} contribuidores.
```

vira

```markdown
🚀 Este projeto tem 361 commits de 2 contribuidores.
```

O GitScope não impõe layout. Texto, títulos, ordem, emoji, tabelas, badges, listas e HTML são todos seus; você decide onde ficam os valores.

## Início rápido

1. Envolva num bloco a parte do README que o GitScope pode alterar:

   ```markdown
   ## Estatísticas

   <!-- gitscope:start -->

   Início do projeto: {first_commit}

   Último commit: {last_commit}

   Total de commits: {commits}

   Contribuidores: {contributors}

   <!-- gitscope:end -->
   ```

2. Rode, na raiz do repositório:

   ```bash
   gitscope update
   ```

3. Faça o commit do resultado. Rode `gitscope update` de novo sempre que quiser números novos; se nada mudou, o arquivo não é tocado.

Nada fora do bloco é modificado, então o resto do README continua 100 % seu.

## Os três comandos

| Comando | O que faz |
|---|---|
| `gitscope update [FILE]` | Reescreve os blocos de `FILE` **no lugar** (padrão: `README.md` na raiz do repositório). |
| `gitscope markdown FILE` | Imprime `FILE` com as variáveis resolvidas. **Nunca modifica `FILE`.** Use `-o OUT` para gravar em outro arquivo (`-o -` é o stdout). |
| `gitscope variables` | Lista todas as variáveis com uma descrição. |

`update` e `markdown` aceitam as opções de seleção de sempre, que mudam os números: `--repo PATH_OR_URL` (padrão `.`, então uma URL também serve), `--since`, `--until`, `--author`, `--branch`, além de `--keep-unknown` e `-v`.

```bash
gitscope update                                     # README.md do repositório atual
gitscope update docs/STATS.md --since 2026-01-01
gitscope update README.md --repo ../outro-repo
gitscope markdown README.template.md -o README.md   # mantém um arquivo de template separado
gitscope markdown README.md                         # prévia, nada é gravado
```

### `update` vs. `markdown`

* **`markdown`** é uma função pura: entra o template, sai o documento resolvido. Com blocos, só os blocos são resolvidos; um arquivo **sem nenhum bloco** é resolvido por inteiro, então um `README.template.md` simples também funciona.
* **`update`** exige pelo menos um bloco e reescreve o arquivo. Como os valores substituem as variáveis, o template se perderia e o próximo `update` não teria o que renovar. Por isso o primeiro `update` guarda o template de cada bloco num **comentário HTML invisível** (não aparece no GitHub), seguido dos valores atuais:

  ```markdown
  <!-- gitscope:start -->
  <!-- gitscope:template

  Total de commits: {commits}

  -->

  Total de commits: 361

  <!-- gitscope:end -->
  ```

  O próximo `update` relê esse template e renova os valores com os dados atuais. Rodar duas vezes com os mesmos dados gera arquivos idênticos byte a byte.

  **Para mudar o que um bloco diz, edite o template dentro do comentário** e rode `update`. Edições feitas nos valores visíveis são sobrescritas. Para parar de usar o GitScope num bloco, rode `gitscope markdown FILE -o FILE.new`, que grava os valores sem os templates guardados.

## Variáveis

Rode `gitscope variables` para ver a lista atual. Contagens usam separador de milhar (`85,164`) e as datas são `YYYY-MM-DD` em UTC. Quando um valor não existe (repositório vazio, nenhuma linguagem reconhecida...), ele aparece como `n/a`, e as contagens como `0`.

| Variável | Valor |
|---|---|
| **Repositório** | |
| `{repository_name}` | Nome do repositório |
| `{first_commit}` | Data do primeiro commit |
| `{last_commit}` | Data do último commit |
| `{period}` | `2026-01-03 → 2026-09-18` |
| `{branches}` | Número de branches |
| `{tags}` | Número de tags |
| **Commits** | |
| `{commits}` | Número de commits |
| `{first_commit_message}` | Primeira linha da mensagem do primeiro commit |
| `{last_commit_message}` | Primeira linha da mensagem do último commit |
| `{most_active_hour}` | Hora (UTC) com mais commits, por exemplo `14:00` |
| **Contribuidores** | |
| `{contributors}` | Número de contribuidores |
| `{top_author}` | Contribuidor com mais commits |
| `{top_author_commits}` | Número de commits dele |
| `{top_author_percentage}` | Parcela dele, por exemplo `54.9` (sem o sinal `%`: escreva `{top_author_percentage}%`) |
| `{author:NAME:commits}` | Commits de um contribuidor |
| `{author:NAME:insertions}` | Linhas adicionadas por um contribuidor |
| `{author:NAME:deletions}` | Linhas removidas por um contribuidor |
| **Arquivos** | |
| `{files}` | Arquivos no retrato atual |
| `{insertions}` | Total de linhas adicionadas |
| `{deletions}` | Total de linhas removidas |
| **Linguagens** | |
| `{language_count}` | Número de linguagens reconhecidas |
| `{top_language}` | Linguagem com mais arquivos |

Os números são exatamente os do relatório normal (`gitscope` e `--json`), calculados com as mesmas definições; veja [Como os números são definidos](../README.md#como-os-números-são-definidos). Em particular, seguem os seus filtros `--since`/`--until`/`--author`/`--branch`, exceto *arquivos* e *linguagens*, que descrevem o retrato atual.

### Variáveis compostas: `{author:NAME:FIELD}`

Uma variável pode receber argumentos separados por `:`. `{author:NAME:commits}` encontra um contribuidor pelo nome ou e-mail exato (sem diferenciar maiúsculas) e dá os commits dele; `insertions` e `deletions` funcionam do mesmo jeito.

```markdown
A Alice fez {author:Alice:commits} commits: +{author:Alice:insertions} / -{author:Alice:deletions}.
```

Um autor que não existe é um erro (`no contributor named 'X'`), não um `0` silencioso.

É um primeiro passo: o mesmo mecanismo vai levar variáveis mais específicas depois (veja o [roadmap](../README.md#roadmap)).

## Regras de sintaxe

* Uma variável é `{nome}` ou `{nome:arg:arg}`. O nome começa com letra ou `_` e continua com letras, dígitos ou `_`.
* **Qualquer outra coisa com chaves é deixada como está**: JSON (`{"a": 1}`), CSS (`a { color: red }`), `{}`, `{ com espaço }`, `{duas palavras}`, `${VAR_DO_SHELL}` e `{{mustache}}`.
* Para escrever um `{commits}` literal, escape com uma barra invertida: `\{commits}` imprime `{commits}`. Uma barra invertida antes de qualquer coisa que não seja uma variável fica como está.
* Variáveis funcionam em qualquer lugar da região processada: parágrafos, tabelas, citações, listas, títulos, atributos HTML, texto de link.
* Os valores entram **como são, uma única vez**. Uma mensagem de commit que contém `{commits}` aparece como `{commits}`; nunca é resolvida de novo. `<!--` e `-->` dentro de um valor são escritos como `&lt;!--` e `--&gt;`, então uma mensagem de commit não consegue abrir nem fechar um comentário ou um bloco.
* O texto de um valor não é escapado para Markdown: um `|` numa mensagem de commit dentro de uma célula de tabela quebra a tabela. Deixe mensagens fora de tabelas ou escape-as você mesmo.

## Blocos

* Um marcador deve estar **sozinho na linha** (com até três espaços de recuo): `<!-- gitscope:start -->` e `<!-- gitscope:end -->`. Os espaços dentro do comentário são flexíveis (`<!--gitscope:start-->` funciona).
* Marcadores dentro de código com cerca (```` ``` ```` ou `~~~`), em código inline ou em código indentado são **documentação**, não marcadores, então o seu README pode explicar o recurso sem acioná-lo.
* Pode haver **vários blocos**. Cada um é independente.
* Os nomes que o projeto já teve, `gitstatus` e `gitstats`, continuam aceitos (`<!-- gitstatus:start -->`), inclusive o comentário de template escrito por uma versão antiga, que é reescrito com o nome atual no próximo `update`.

## Segurança

O `update` nunca deixa um README pela metade ou corrompido:

* O arquivo inteiro é lido, **validado e renderizado em memória primeiro**. Só se tudo estiver certo ele é gravado.
* Em **qualquer** erro o arquivo original não é modificado.
* A gravação é **atômica**: um arquivo temporário é escrito ao lado do README, descarregado em disco e renomeado sobre ele, então uma execução interrompida deixa o arquivo antigo ou o novo. As permissões são mantidas e um README que seja symlink é seguido, não substituído.
* Se o resultado for igual ao conteúdo atual, nada é gravado.
* Quebras de linha (CRLF ou LF), um BOM UTF-8 e a falta de quebra de linha final são preservados. As linhas que o GitScope acrescenta usam a quebra de linha do próprio arquivo.
* `markdown -o OUT` se recusa a sobrescrever o template que está lendo.

O que é validado (as mensagens do programa são em inglês):

| Problema | Mensagem (resumida) |
|---|---|
| Início sem fim | `line 12: <!-- gitscope:start --> is never closed` |
| Fim sem início | `line 30: found <!-- gitscope:end --> without a matching <!-- gitscope:start -->` |
| Início duplicado ou aninhado | `line 20: <!-- gitscope:start --> found inside the block opened at line 12` |
| Nenhum bloco (no `update`) | `no GitScope block found: wrap the part to update in ...` |
| Variável desconhecida | `Unknown GitScope variable: {comits} (line 14)` + `Did you mean {commits}?` + a lista das variáveis disponíveis |
| Variável mal usada | `Invalid GitScope variable: {commits:1} (line 9): {commits} takes no arguments` |
| Template guardado sem fechamento | `line 14: the stored template comment ... is never closed` |
| `-->` no template de um bloco (no `update`) | o bloco não pode ser guardado dentro de um comentário HTML; use `markdown -o` com um arquivo de template separado |

Todos os problemas de variáveis do arquivo são reportados juntos, com o número da linha.

### Avisos em vez de erros

`--keep-unknown` deixa variáveis desconhecidas ou mal usadas exatamente como foram escritas, imprime um aviso para cada uma no stderr e resolve todo o resto:

```console
$ gitscope update --keep-unknown
gitscope: warning: left {my_custom_thing} untouched (line 14)
Updated /path/to/README.md (1 block, 5 variables)
```

## Exemplos

Todos estes funcionam, porque o GitScope só troca as variáveis:

```markdown
Início: {first_commit}

Início do projeto → {first_commit}

> Projeto iniciado em {first_commit}

**Projeto iniciado em {first_commit}**

🚀 Este projeto tem {commits} commits de {contributors} contribuidores.

| Métrica | Valor |
|---|---:|
| Commits | {commits} |
| Contribuidores | {contributors} |
| Arquivos | {files} |

<p align="center"><b>{commits}</b> commits · <b>{contributors}</b> contribuidores</p>

```

### Mantendo um arquivo de template separado

```bash
gitscope markdown README.template.md -o README.md
```

O `README.template.md` guarda as variáveis para sempre; o `README.md` é gerado e só contém valores. Não precisa de blocos, já que um arquivo sem blocos é resolvido por inteiro.

## Estendendo

Acrescentar uma variável é uma entrada em `standard_variables()`, em [`src/output/variables.rs`](../src/output/variables.rs): um nome, um grupo, uma descrição e uma função resolvedora. Os resolvedores recebem os argumentos escritos depois do nome, então as variáveis compostas usam o mesmo mecanismo. Mais nada precisa mudar; o `gitscope variables`, as mensagens de erro e as sugestões passam a incluir a nova variável automaticamente. Veja [ARCHITECTURE.md](ARCHITECTURE.md).
