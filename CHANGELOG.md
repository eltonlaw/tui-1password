# latest

- Fix issue where item details with a field that doesn't have a value gets filtered out, messing up the indexing. Yanking from that row would also cause a panic.
- Pressing `R` reloads the item/item details (calls the CLI again)
- Searching with `/<pattern>` and then `<CR>` goes to the first instance of `<pattern>` anywhere in the title column. Pressing `n` goes to the next match, wrapping back to the first match when you hit the end of the matches list.

# 0.1.1

- Config file implementation. read `AppConfig` struct out of some `tui-1password.yaml`
- G to nav to bottom
- clipboard option allows args
- added all the available headers for `Item` and `ItemDetais` (everything in the 1password json response)
- On login if token file expired or doesn't exist, will ask for password and pass to `op signin`

# 0.1.0

- navigate up and down with j,k (or Ctrl-U, Ctrl-D)
- press enter to see hidden info
- yank into clipboard with y
- sort by column
