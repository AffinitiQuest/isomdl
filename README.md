# isomdl

ISO mDL implementation in Rust

## CLI tool

This crate contains a CLI tool. Run the `--help` command to see what actions you can perform.

```bash
cargo run -- --help
```

For example you can verify a supplied mdl and dump out it's claims:
```bash
cargo run -- verify test/stringified-mdl.txt
```

Also, issue a new mdl from a supplied json description
```bash
cargo run -- issue mdl.json
```