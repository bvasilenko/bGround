# bground

Prompt lookup tool. Agent names a claim type from a fixed list of 16; bground returns the prompt for that claim type. The prompt tells the agent how to check the claim against supplied evidence.

Built for agentic loops. Parses a typed claim, takes supplied evidence, writes a verdict on stdout, exits with a discriminating code so the calling agent can branch.

```
bground verify <claim>            check a claim against supplied evidence; exit 0 / 1 / 2 / 64
bground claim-types               list the 16 supported claim-type identifiers
bground init                      scaffold a manifest in the current directory
bground update                    self-update to the latest published version
bground tail                      stream recent verdict transcripts
bground explain                   print taxonomy + exit-code reference
```

Exit code contract: `0` claim grounded, `1` claim NOT grounded, `2` internal error, `64` malformed input.

## Install

```sh
cargo install --git https://github.com/bvasilenko/bGround
```

## Use

```sh
bground verify "file-exists:README.md:README must exist" \
    --evidence readme=$(test -f README.md && echo present || echo absent)
# stdout: GROUNDED
# exit: 0

bground verify "value-equals:VERSION:0.1.0" \
    --evidence VERSION=0.2.0
# stdout: UNGROUNDED
# exit: 1
```

The `<claim>` argument is parsed as `<claim-type>:<target>:<assertion>`. Supply evidence with repeatable `--evidence <id>=<value>` flags. Optional flags: `--manifest <path>`, `--json`, `--quiet`, `--reason <text>`.

## Claim taxonomy

Closed 16-variant `ClaimType` enum. The taxonomy is fixed at this version; widening lands in a later version.

| Category | Variants |
|---|---|
| Existence | `file-exists`, `fn-defined`, `dependency-installed` |
| Equality | `value-equals`, `state-equals`, `fn-signature` |
| Runtime | `fn-return-type`, `url-returns`, `cmd-output-matches`, `behavior` |
| Coherence | `coherent-refactor`, `coherent-migration`, `coherent-spec-impl`, `coherent-contract`, `coherent-test-cover`, `coherent-dep-upgrade` |

`bground claim-types` prints the full list.

## License

MIT.
