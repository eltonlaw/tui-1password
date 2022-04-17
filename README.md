# 1password-tui

Interactive interface around the 1password CLI

## Install

### From source

    cargo build --release
    sudo mv ./target/release/tui-1password $HOME/.local/bin

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
