<p align="center"><img src="assets/brand/png/icon-128.png" width="128" height="128" alt=""></p>

# lesefrucht

<!-- givn:begin:coverage-badge -->
[![Line Coverage: n/a](https://img.shields.io/badge/line%20coverage-n--a-brightgreen)](coverage/cobertura-coverage.xml)
[![Branch Coverage: n/a](https://img.shields.io/badge/branch%20coverage-n--a-brightgreen)](coverage/cobertura-coverage.xml)
<!-- givn:end:coverage-badge -->

lesefrucht turns a private calibre library into a citable source: a reader, or the
agent acting for them, asks a question about the books and receives a response in
which every sentence carries a passage from the library, with a citation that
carries the exact position and opens its edition. It is for the person who owns
and maintains that library.

## Quick start

The repository is specification-first: the Gherkin specification and the
architecture record are authoritative, and the Rust implementation is built from
them.

```sh
git clone <this repository>
cd lesefrucht
nix develop
cargo build
./dev-up.sh          # product, deterministic embedding twin, fixture library
./run-tests.sh       # non-e2e scenarios
./run-tests.sh --e2e # the @e2e scenarios through the compose stack
```

What the first change delivers is in
[`docs/arc42/01-introduction-and-goals.md`](docs/arc42/01-introduction-and-goals.md).


## Prerequisites

| Requirement | Why |
|---|---|
| Rust stable, edition 2024 | the product language (`docs/arc42/02-architecture-constraints.md`); the repository pins it through its Nix development shell |
| calibre, providing the `calibredb` command | the documented library interface: the product lists with `calibredb list` and exports edition bytes with `calibredb export` |
| A calibre content-server URL or a local library path | the library target passed to `calibredb --with-library` (`library.url`) |
| An OpenAI-compatible embedding endpoint | passage and claim embeddings; required configuration with no default |
| Nix | the development shell resolves Rust, calibre and the coverage tools |
| podman and podman-compose, for the local and e2e test stacks | local and end-to-end tests run in rootless containers; production does not need a container runtime |

## Usage

The product exposes the `passage-retrieval` capability over MCP at
`http://127.0.0.1:8787/mcp` with two tools:

- `search_passages` takes a claim or phrase, a language, an optional scope
  (genre, language, authors, year range, named works) and an optional limit; it
  returns up to twenty candidate passages, each with a citation, a score and the
  expansion used, or a distinguishable `no evidence` result, or an unavailability
  report.
- `read_passage` takes a citation and an optional `window_chars` and returns the
  surrounding text with exact character bounds.

## Contributing

Work runs through the givn workflow: propose, spec, design, design-review, tasks,
implement, review, archive. Each Gherkin scenario is one atomic commit whose
subject is the release note and whose body names the scenario. Do not edit the
state under `givn/` by hand.

## License

MPL-2.0 — see [`LICENSE`](LICENSE).
