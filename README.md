# bground

CLI claim-grounding checker. Reads claim plus evidence; emits proceed-or-stop directive.

## Install

```sh
cargo install --git https://github.com/bvasilenko/bGround
```

Once published:

```sh
cargo install bground
```

## Use

```sh
bground verify "file-exists:README.md:README exists" --evidence readme=present
```

## License

MIT.
