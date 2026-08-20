# rstats-cli

Install the native Rust system monitor with npm:

```sh
npm install --global rstats-cli
rstats --monitor
```

You can also run it without a global install:

```sh
npx rstats-cli --monitor
```

Useful commands:

```sh
rstats --help
rstats --monitor
rstats --open-recordings
```

The package installs one optional native package matching your operating system and CPU architecture. It does not download binaries during installation or execution.

Supported targets are macOS x64/arm64, Linux x64/arm64, and Windows x64. Linux requires `xdg-open` for `--open-recordings`.
