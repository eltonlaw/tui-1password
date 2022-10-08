# 1password-tui

Interactive interface around the 1password CLI

## Install

### From source

    cargo build --release
    sudo mv ./target/release/tui-1password $HOME/.local/bin

## Usage

Navigation

    Up Arrow / `k`  :   Up a row
    Down Arrow / `j`:   Down a row
    C-d                 Down 6 rows
    C-u                 Up 6 rows
    `q`:                Quit
    `:`:                Open cmd mode
    `y`:                Yank to clipboard either the selected title in list
                        view or whatever field value is highlighted in item view

Available commands

    :q
    :qa
    :sort id
    :sort id asc
    :sort id desc
    :sort updated_at
    :sort updated_at asc
    :sort updated_at desc
    :sort title
    :sort title asc
    :sort title desc


## Configuration

A configuration file is looked for in the following order. If none of these exist, it will loop back to the top and try to create that file in each directory. Wherever it lands, the parent dir of the config file is the app root directory.

- `${TUI_1PASSWORD_HOME}/tui-1password.yaml`
- `${XDG_CONFIG_HOME}/tui-1password/tui-1password.yaml`
- `${HOME}/.tui-1password/tui-1password.yaml`

Sample `tui-1password.yaml` file. This is still a bit of a WIP, so every available option has to be set.

    ---
    headers:
      - id
      - title
      - updated_at
    root_dir: /home/eltonlaw/.config/tui-1password
    debug: false
    clipboard_bin: wl-copy

`headers`: The columns used in the top level item list view. Defaults to `[id title updated_at]`. You can add any of the properties from `struct ItemListEntry`.

`clipboard_bin`: Some clipboard copy binary that you can pipe a string into. On mac this would be `pbcopy` and on some linux systems I think this would be `xsel -ib`

`root_dir`: This should just be the parent dir of the config file. A bit redundant, and will be unnecessary in the future.

`debug`: Debug flag. Doesn't do much at the moment.

## Quickstart

This is a tui wrapper around the 1password CLI, so [that needs to be installed first](https://1password.com/downloads/command-line/). Pipe the token to a file

	mkdir -p ~/.tui-1password
    op signin my > ~/.tui-1password/token

The `token` file looks something like this, it's just a shell script exporting an environment var

    export OP_SESSION_QAXFAARFSVGTTOAHF37M76FDT4="AVpOk3jBbp-EcTxoEyYdwFVMPIrSZII4MZdngyq9MFv"
    # This command is meant to be used with your shell's eval function.
    # Run 'eval $(op signin my)' to sign in to your 1Password account.
    # Use the --raw flag to only output the session token.

The env var is read and used to invoke CLI commands and the returned JSON is thrown into the interface. Session tokens expire after 30 min so this will need to be run again.

![Item List](https://github.com/eltonlaw/tui-1password/blob/main/imgs/itemlist.png?raw=true)

- To see a specific entry, `<Enter>` shows item detail view
- When in item detail view `q` to go back.
- To sort by a column use `:sort`, ex. `:sort updated_at` or `:sort updated_at desc`. Default is `:sort title asc`
- To quit press `q`. Vim bindings `:q`<Enter> and `:qa`<Enter> also work

By default `--cache` is passed to the `op` CLI.
