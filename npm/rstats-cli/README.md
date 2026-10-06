# rstats-cli

Install the native Rust system monitor with npm:

```sh
npm install --global rstats-cli
rstats --monitor
```

![rstats dashboard](https://raw.githubusercontent.com/PrakashSewani/rstats-cli/main/docs/screenshots/dashboard-dark.png)

You can also run it without a global install:

```sh
npx rstats-cli --monitor
```

Useful commands:

```sh
rstats --help
rstats --monitor
rstats --once            # print a single snapshot and exit
rstats --watch           # stream plain-text lines until interrupted
rstats --export <file>   # export a recording to CSV or JSON
rstats --report <file>   # summarize a recording
rstats --open-recordings
```

Pick a color theme with `--theme dark|light|mono`, or press `t` while the monitor runs to cycle themes live.

The package installs one optional native package matching your operating system and CPU architecture. It does not download binaries during installation or execution.

Supported targets are macOS x64/arm64, Linux x64/arm64, and Windows x64. Linux requires `xdg-open` for `--open-recordings`.
