# cargo playground

Cargo playground opens a local playground in the editor of your choice.

## Install

You can install it directly using cargo
```
$ cargo install --git https://github.com/Lutetium-Vanadium/cargo-playground
```

Or clone the project and then install
```
$ git clone https://github.com/Lutetium-Vanadium/cargo-playground
$ cd cargo-playground
$ cargo install --path .
```

> `cargo-playground` on crates.io was already registered to someone else
> as a dummy project, and is not this project. Please install only using
> the above methods.

## Usage

A few examples:
```
# Create a new playground
$ cargo playground new -n test-playground rand

# List existing playgrounds
$ cargo playground ls

# Open an existing playground
$ cargo playground open -e vim -a=-o2 test-playground
```

#### Environment variables

The following environment variables are used:

- `VISUAL`: The editor in which to open the playground. It is required
  unless --editor is given

- `CARGO_PLAYGROUND_DIR`: The path to the directory which is used for
  creating playgrounds. If not set, the temp directory is used. eg: in
  unix: `/tmp/cargo-playground/`

#### `cargo playground new`

This creates and opens a new playground.

It has the following options and flags:
```
 -g, --gui             Indicates the editor is a GUI based
 --no-w                Do not pass -w flag when opening GUI editor
 -n, --name <name>     The name of the playground. By default it is
                       generated from the current timestamp
 -e, --editor <editor> The editor to open the project in. By default it
                       is taken from the VISUAL env variable.
 -a, --args <args>...  Args to be given to be supplied to the editor.
```

It also takes a list of dependencies in the following formats:
1. `<dep-name>`                  eg: `"rand"`
2. `<dep-name> = <dep-version>`  eg: `"rand = 0.7"`
   > `<dep-version>` should not have quotes

#### `cargo playground open`

This opens a playground with the given name.

It has the following options and flags:
```
 -g, --gui             Indicates the editor is a GUI based
 --no-w                Do not pass -w flag when opening GUI editor
 -e, --editor <editor> The editor to open the project in. By default it
                       is taken from the VISUAL env variable.
 -a, --args <args>...  Args to be given to be supplied to the editor.
```

#### `cargo playground ls`

This lists all the playgrounds.

#### `cargo playground clean`

Deletes the Cargo projects in the playground directory.

It has the following options:
```
 -m, --matches <matches> A regex to match against playground names. If
                         not given all will be deleted.
```

## Support

- Terminal based editor + tmux: It opens a pane to the right which has
  the output process, and the editor will be open to the left. When the
  editor is closed, the playground will be closed.

- GUI based editors: This requires the extra flag `-g/--gui`. It shows
  the output process in the main terminal and opens the editor.

  > Any output from the editor to stdout/stdin will be ignored and not
  > shown.

  For example:
  ```
  # Creates a new playground with vscode as editor
  $ cargo playground new -ge code
  #                        ^^^^^^-- not required if VISUAL is set to code
  ```

  > I have noticed that cli openers for gui programs exit instantly and
  > require a `-w` flag to make it wait. As such opening with gui
  > editors automatically supplies a `-w` flag. In case your editor
  > doesn't behave this way, use the `--no-w` flag to disable it.

If you want to add specific for some editor or environment feel free to
open a PR!

#### Credits to Sergey Potalov and his post [Rust Playground At Your Fingertips](https://www.greyblake.com/blog/2021-03-12-rust-playground-at-your-fingertips/) for the inspiration!
