# waypoint

`waypoint` is an interactive shell wrapper that routes commands to different working directories based on a YAML config.

It is useful when you work across multiple repos/packages and want commands like `bun`, `cargo`, etc. to run from the right directory automatically.

---

## What it does

When you type a command in the waypoint shell:

1. waypoint parses the command (`cmd`) and its arguments.
2. It loads `waypoint.yaml`.
3. It finds config entries for that command.
4. It picks the first entry whose configured `args` all match your typed args.
5. If matched, it runs the command from that entry's `dir`.
6. If no entry matches, it runs from the current directory.

It also provides built-in `cd` behavior and a couple of helper commands.

---

## Installation

## 1) Cargo

```bash
cargo install waypoint
```

## 2) Install script (Unix/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/hazyio/waypoint/main/install.sh | sh
```

Update existing install:

```bash
curl -fsSL https://raw.githubusercontent.com/hazyio/waypoint/main/install.sh | sh -s -- update
```

Optional environment variables:

- `INSTALL_DIR` (default: `~/.local/bin`)
- `WAYPOINT_REPOSITORY` (default: `hazyio/waypoint`)

## 3) Install script (Windows PowerShell)

```powershell
iwr https://raw.githubusercontent.com/hazyio/waypoint/main/install.ps1 -OutFile install.ps1
./install.ps1
```

Update existing install:

```powershell
iwr https://raw.githubusercontent.com/hazyio/waypoint/main/install.ps1 -OutFile install.ps1
./install.ps1 update
```

Optional environment variables:

- `INSTALL_DIR`
- `WAYPOINT_REPOSITORY` (default: `hazyio/waypoint`)

## 4) Download from GitHub Releases

Download a prebuilt archive from:

- <https://github.com/hazyio/waypoint/releases/latest>

Supported prebuilt targets:

- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `aarch64-apple-darwin`

---

## Usage

Start waypoint:

```bash
waypoint
```

Inside the shell:

- `exit` → leave waypoint
- `waypoint` → prints environment confirmation
- `waypoint-update-config` → set config path to `<current dir>/waypoint.yaml`
- `cd`, `cd -`, `cd ~` → built-in directory changes

All other lines are executed by the platform shell (`sh -c` on Unix, `cmd /C` on Windows).

---

## CLI flags

```text
-c, --config <PATH>      Path to config file (default: ./waypoint.yaml)
    --continue-on-error  If config loading fails, skip executing that command line and continue the shell loop
```

Examples:

```bash
waypoint --config /path/to/waypoint.yaml
waypoint --continue-on-error
```

---

## Configuration (`waypoint.yaml`)

Config is a top-level map of named settings:

```yaml
bun:
  cmd: bun
  dir: .

bun-add:
  cmd: bun
  dir: .
  args:
    - add
```

### Fields

Each entry supports:

- `cmd` (string, required): command name to match (`bun`, `cargo`, etc.)
- `dir` (path, required): directory where command should run
  - relative paths are resolved from the config file directory
  - must exist and must be a directory
- `args` (list of strings, optional): argument tokens required for this rule

### Matching behavior

- Rules are first filtered by `cmd`.
- For `args`:
  - If config `args` is empty, it matches **only** when command args are empty.
  - If config `args` is non-empty, **all** configured args must be present in the typed args.
  - Extra typed args are allowed.

Example:

```yaml
cargo:
  cmd: cargo
  dir: ../rust

cargo-build:
  cmd: cargo
  dir: ../rust
  args: [build]
```

- `cargo` → matches `cargo` rule (empty args)
- `cargo build` → matches `cargo-build`
- `cargo build --release` → matches `cargo-build`

---

## Safety rule

waypoint blocks command lines containing:

- `rm -rf /`

---

## Logging

waypoint logs to a daily rotating file in the platform cache directory under `waypoint/waypoint.log`.

You can override log level with `RUST_LOG`, for example:

```bash
RUST_LOG=debug waypoint
```

---

## Development

Run checks locally:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```
