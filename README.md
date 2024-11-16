# user-startup-py

Just auto run command on startup. Support Windows, Linux and Macos, no root permission required.

## Installation

- Download from [Releases](https://github.com/lxl66566/user-startup-rs/releases)
- use [bpm](https://github.com/lxl66566/bpm)
  ```bash
  bpm i lxl66566/user-startup-rs
  ```
- use [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)
  ```bash
  cargo binstall user-startup
  ```
- use cargo (compile from source)
  ```bash
  cargo install user-startup
  ```

## Usage / Examples

```bash
# Add a new startup command
user-startup add 'my command'

# List all startup commands and their ids
user-startup list

# Remove startup commands (by id)
user-startup remove 'my'

# Open the startup folder
user-startup open
```

to see more Usage, run `user-startup -h`.

## Use as lib

```toml
[dependencies]
user-startup = { version = "0.1.0", default-features = false }
```

see [tests](tests/intergration_test.rs).

## QA

- **Q**: If I have more than one line command, how to run them?
  - **A**: Manually write the commands to a script (.bat, .ps1, .sh ...), and runs this script as a command.

## Thanks

- [typicode/user-startup](https://github.com/typicode/user-startup)
