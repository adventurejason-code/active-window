# active-window

A minimal Rust CLI that prints the currently active (focused) Wayland window
title to stdout by querying the
[`zwlr_foreign_toplevel_manager_v1`](https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1)
protocol.

## Usage

```
active-window              # prints focused window title
active-window --json       # prints {"title":"...","app_id":"..."}
active-window --help
```

Exit code `0` on success, `1` if no active window found or on error.

## Compositor support

Any compositor that exposes the
[`zwlr_foreign_toplevel_manager_v1`](https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1)
protocol will work.

## Installation (Arch Linux)

```bash
curl -O https://raw.githubusercontent.com/adventurejason-code/active-window/main/PKGBUILD
makepkg -si
```

`makepkg` will fetch the source from GitHub, build a stripped release binary,
and install it to `/usr/bin/active-window`.

### Manual build

```bash
git clone https://github.com/adventurejason-code/active-window.git
cd active-window
cargo build --release
./target/release/active-window
```

## How it works

The tool connects to the Wayland socket, binds
`zwlr_foreign_toplevel_manager_v1`, performs three synchronous roundtrips to
collect the full set of open window handles and their property events, then
finds the handle whose `state` array includes the `activated` (value `2`) enum
entry and prints its title.