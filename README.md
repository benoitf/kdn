# kdn

CLI to manage [Kaiden](https://github.com/openkaiden) AI agent workspaces from the terminal.

It talks to the API server of a locally running Kaiden desktop app: list and delete workspaces, open a shell inside one, or attach to the agent session shared with the Kaiden UI.

## Requirements

- Linux or macOS (raw terminal mode uses termios; the crate refuses to build elsewhere)
- Rust stable toolchain
- Kaiden running locally with the `api.server.enabled` preference turned on

## Install

From source:

```sh
cargo install --path .
# or
make install
```

## Usage

```sh
kdn workspace list                       # alias: kdn ws ls
kdn ws ls -o json                        # raw JSON from the API
kdn ws delete <name>                     # alias: rm
kdn ws terminal <name> [command...]      # alias: connect; default command is /bin/sh
kdn ws agent <name>                      # attach to the agent session shared with the Kaiden UI
```

`--gateway <NAME>` can be passed to any `workspace` subcommand to target a gateway other than the active one.

`terminal` and `agent` exit with the exit code of the remote command.

## How it finds Kaiden

Kaiden publishes the API port and a bearer token in an `api-port` file (first line port, second line token). `kdn` looks for it in this order:

1. `$KAIDEN_HOME_DIR/api-port`
2. `$XDG_DATA_HOME/kaiden/api-port`
3. `~/.local/share/kaiden/api-port`

All traffic goes to `127.0.0.1` over plain HTTP and WebSocket.

## Development

```sh
make build       # debug build
make test        # unit tests
make lint        # rustfmt check + clippy
make ci-checks   # lint + test, what CI runs on pull requests
make release     # optimized build in target/release/kdn
```

See `CONTRIBUTING.md` for the contribution workflow and `AGENTS.md` for the code layout.

Releases are built by [cargo-dist](https://opensource.axo.dev/cargo-dist/) when a `vX.Y.Z` tag is pushed. See `AGENTS.md` for the release steps.

## License

Apache-2.0. See [LICENSE](LICENSE) and the SPDX headers in each source file.
