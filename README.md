# 1password-tui

Interactive interface around the 1password CLI

## Quickstart

Since this is just a wrapper around the 1password CLI, please [install](https://1password.com/downloads/command-line/) that first and then login, piping the token to a file

	mkdir -p ~/.tui-1password
    op signin my > ~/.tui-1password/token

Session tokens expire after 30 min so you will need to log back in after
