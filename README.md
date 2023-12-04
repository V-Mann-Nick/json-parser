# JSON Parser in Rust

JSON parser compliant with the JSON specification ([RFC 7159](https://datatracker.ietf.org/doc/html/rfc7159)).

It features:

- Support for numbers written in scientific notation (e.g. `2e10`)
- Representation of numbers in the most suitable number type `i64`, `u64` or `f64` (no arbitrary precision supported!)
- Support for escape sequences (`\n`, `\r`, `\f`, …)
- Support for Unicode escape sequences (`\u005C`) as well as UTF-16 surrogate pairs (`\uD834\uDD1E`) for characters not in the basic multilingual plane
- Serialization back to JSON from Rust representation

## How to run

This project uses the [nix package manager](https://github.com/NixOS/nix) using [nix flakes](https://nix.dev/concepts/flakes.html) for providing the rust toolchain and building packages, as well as [direnv](https://direnv.net/) for automatic shell loading.

```sh
git clone git@github.com:V-Mann-Nick/json-parser.git
cd json-parser

# You need both direnv and nix configured for flakes for this to work
direnv allow  # will load the nix dev shell with rust toolchain

# Build with cargo
cargo build --release
./target/release/qj --help

# Build with nix
nix build

# Install into nix profile
nix profile install .

# Build OCI image
nix build .#image
```

## References

- [Things Programmers Can Do in One Week | Blog | build-your-own.org](https://build-your-own.org/blog/20231108_1week/?id=20231108)
- [GitHub - serde-rs/json: Strongly typed JSON library for Rust](https://github.com/serde-rs/json)
- [GitHub - jqlang/jq: Command-line JSON processor](https://github.com/jqlang/jq)
- [JSON standard - floating point numbers - Stack Overflow](https://stackoverflow.com/a/19554986/15782961)
