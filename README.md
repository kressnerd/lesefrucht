<p align="center"><img src="assets/brand/png/icon-128.png" width="128" height="128" alt=""></p>

# lesefrucht

<!-- givn:begin:coverage-badge -->
[![Line Coverage: n/a](https://img.shields.io/badge/line%20coverage-n--a-brightgreen)](coverage/cobertura-coverage.xml)
[![Branch Coverage: n/a](https://img.shields.io/badge/branch%20coverage-n--a-brightgreen)](coverage/cobertura-coverage.xml)
<!-- givn:end:coverage-badge -->

lesefrucht turns a private calibre library into a citable source: a reader, or the
agent acting for them, asks a question about the books and receives a response in
which every sentence carries a passage from the library, with a citation that opens
the exact position. It is for the person who owns and maintains that library.

## Quick start

The repository is specification-first. It holds the Gherkin specification, the
architecture record and the change in flight, but no product code yet, so a clone
builds nothing.

```sh
git clone <this repository>
cd lesefrucht
```

What the first change delivers is in
[`docs/arc42/01-introduction-and-goals.md`](docs/arc42/01-introduction-and-goals.md).


## Prerequisites

| Requirement | Why |
|---|---|
| Rust stable, edition 2024 | the product language (`docs/arc42/02-architecture-constraints.md`) |
| A calibre library served by the calibre content server | the only supported library interface |
| A host without a container runtime | the service runs directly under systemd |

## Usage

No binary exists yet. When the first change lands, an agent calls the MCP
`find-passages` capability with a claim or phrase and receives up to twenty
candidate passages, each with a citation, a score and the expansion used, or a
distinguishable `no evidence` result.

## Contributing

Work runs through the givn workflow: propose, spec, design, design-review, tasks,
implement, review, archive. Each Gherkin scenario is one atomic commit whose
subject is the release note and whose body names the scenario. Do not edit the
state under `givn/` by hand.

## License

MPL-2.0 — see [`LICENSE`](LICENSE).
