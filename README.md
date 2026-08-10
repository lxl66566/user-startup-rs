# user-startup-rs

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

# Add a multi-line command or one using shell syntax (pipes, &&, ...)
user-startup add --shell 'echo a && echo b'

# Add a command even if it already exists
user-startup add --force 'my command'

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
user-startup = { version = "0.4", default-features = false }
```

see [tests](tests/integration_test.rs) for more examples.

## QA

- **Q**: If I have more than one line command, how to run them?
  - **A**: Use `user-startup add --shell '...'`, or manually write the commands to a script
    (.bat, .ps1, .sh ...) and run this script as a command.
- **Q**: On Linux, my command only starts after I log in. How to run it at boot?
  - **A**: Enable lingering for your user: `loginctl enable-linger $USER`.
- **Q**: Can I use a custom startup directory?
  - **A**: Set the `USER_STARTUP_CONFIG_DIR` environment variable to override
    the platform default.

## Thanks

- [typicode/user-startup](https://github.com/typicode/user-startup)
