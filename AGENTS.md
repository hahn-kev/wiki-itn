# AGENTS.md — wiki-itn

## What this is

Rust CLI that turns Wikipedia’s **In the news** template HTML into an Atom feed, packaged as a Docker/Podman image with cron + nginx.

- Input: HTML from `https://en.wikipedia.org/wiki/Template:In_the_news` (piped on stdin)
- Output: Atom XML on stdout (container writes `/var/www/html/feed.xml`)
- Entry points: `src/main.rs` (`process_html`), `run_wiki_itn.sh`, `entrypoint.sh`

## Layout

| Path | Role |
|------|------|
| `src/main.rs` | Parse HTML → Atom (`process_html`); also the binary `main` |
| `src/news_item.rs` | `NewsItem` struct |
| `tests/integration_test.rs` | Live fetch + generate `feed.xml` |
| `itn-page.html` | Saved HTML fixture for offline `cat … \| wiki-itn` |
| `Dockerfile` | Multi-stage: `cargo test` + release build, then nginx runtime |
| `.github/workflows/docker-ci.yml` | Build image, wait healthy, run feed script, assert `feed.xml` |

`Cargo.toml` exposes the same `src/main.rs` as both lib (`wiki_itn`) and bin (`wiki-itn`). Prefer calling `wiki_itn::process_html` from tests.

## How to test

**Preferred (no local Rust):** Podman. On Windows, start the machine first: `podman machine start`.

```bash
# Unit + integration (needs network for the live Wikipedia test)
podman run --rm --network=host -v "$PWD:/usr/src/wiki-itn:Z" -w /usr/src/wiki-itn rust:latest \
  bash -c 'apt-get update -qq && apt-get install -y -qq curl >/dev/null && cargo test'

# Image build (Dockerfile runs cargo test, but only if tests/ is copied — keep that COPY)
podman build -t wiki-itn:ci-test-image .

# CI-style smoke (after build)
podman run -d --name test-container wiki-itn:ci-test-image
# wait until healthy / nginx answers /nginx_health
podman exec test-container /usr/local/bin/run_wiki_itn.sh
podman exec test-container sh -c 'test -s /var/www/html/feed.xml'
```

Offline smoke with a fixture: `cat itn-page.html | ./target/release/wiki-itn`

## Fragile contract: Wikipedia HTML

`find_root_element` must locate the ITN story list under `#mw-content-text` → `.mw-parser-output`.

Wikipedia has moved to **Parsoid** markup. Expect:

- `.mw-parser-output` may share the div with other classes (`mw-content-ltr mw-parser-output`)
- The news `<ul>` is often nested (e.g. inside `<section>`), not a direct child of parser-output
- Story links may be **absolute** (`https://en.wikipedia.org/wiki/...`) or relative (`/wiki/...`)
- Story items use `<li><b><a …>Title</a></b> …</li>`; footer/ongoing lists usually lack that bold title pattern

When the feed is empty or the binary panics with `Unable to find root element ul`, fetch a fresh page and adjust the locator — do not assume the 2022-era flat `ul` child layout.

Update `itn-page.html` when the live structure changes so offline checks stay meaningful.

## Windows / container pitfalls

- Keep shell scripts (`entrypoint.sh`, `run_wiki_itn.sh`) as **LF**. CRLF breaks `#!/bin/sh` inside Linux images (`No such file or directory`). `.gitattributes` should force LF on `*.sh`.
- Dockerfile must `COPY tests ./tests` before `cargo test`, or CI only runs empty unit suites.
- Rootless Podman OCI images ignore `HEALTHCHECK` unless built with `--format docker`. Curl `http://localhost/nginx_health` inside the container to verify readiness.
- Dockerfile strips `\r` from shell/nginx configs at build time so Windows checkouts still produce a bootable image.

## Change guidance

- Prefer small parser changes with fixture coverage for both legacy and Parsoid shapes.
- Do not commit secrets; `Cargo.lock` is gitignored in this repo today.
- Only commit when asked. Production breakage is usually parser HTML drift or an empty `feed.xml` from a panicked binary.
