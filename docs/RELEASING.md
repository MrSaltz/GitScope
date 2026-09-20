# Como publicar uma versão

**Português** · [English](RELEASING-EN.md)

Este guia é para quem mantém o projeto. A publicação usa o workflow `.github/workflows/release.yml`: ao enviar uma tag `vX.Y.Z`, ele compila os binários, cria a Release no GitHub e anexa os arquivos e o `SHA256SUMS`.

## Antes da primeira publicação (uma vez)

1. Envie o código para o repositório do GitHub (`repository` e `homepage` já estão no `Cargo.toml`).
2. Confira em *Settings → Actions → General* que os workflows podem escrever no repositório (o workflow já pede `contents: write`; organizações podem restringir isso).
3. No crates.io, crie a conta e um token de API e rode `cargo login`. Confira de novo que o nome está livre com `cargo search gitscope`.

## Preparando a versão

1. Escreva a seção `## [x.y.z] - data` em `CHANGELOG.md` e em `CHANGELOG-EN.md`.
2. Mude `version` no `Cargo.toml`. O `cargo build` atualiza o `Cargo.lock`. O cabeçalho do relatório mostra a versão, então procure `GITSCOPE v` nos READMEs e regenere os exemplos se ela aparecer.
3. Rode as verificações: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo test --no-default-features` e `cargo publish --dry-run`.
4. Faça o commit, leve para a `main` e espere o CI ficar verde.

## Publicando

```bash
git tag v1.0.0
git push origin v1.0.0
```

O workflow confere que a tag é igual à versão do `Cargo.toml`, compila quatro binários (Linux x86_64, Windows x86_64, macOS arm64 e x86_64; Linux e macOS com o OpenSSL embutido) e cria a Release com as notas geradas, os arquivos e o `SHA256SUMS`. Tags com hífen (como `v1.1.0-rc.1`) viram pré-releases. Depois de conferir a Release, publique no crates.io:

```bash
cargo publish
```

Deixe o crates.io por último: uma versão publicada lá não pode ser apagada, só marcada como `yank`.

## Testando o workflow

O workflow foi escrito sem que fosse possível executá-lo no GitHub, então a primeira tag é o teste de verdade. Faça o primeiro com um candidato: ponha `version = "1.0.0-rc.1"` no `Cargo.toml`, envie a tag `v1.0.0-rc.1` e confira os quatro arquivos, o `SHA256SUMS` e se o binário roda. Depois apague a tag e a Release do candidato.

## Se algo falhar

Corrija, apague a tag e a Release e envie a tag de novo:

```bash
git push --delete origin v1.0.0
gh release delete v1.0.0 --cleanup-tag
```
