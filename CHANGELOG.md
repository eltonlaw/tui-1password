# latest

- Fix issue where item details with a field that doesn't have a value gets filtered out, messing up the indexing. Yanking from that row would also cause a panic.
- Pressing `R` reloads the item/item details (calls the CLI again)

# 0.1.1

- Config file implementation. read `AppConfig` struct out of some `tui-1password.yaml`
- G to nav to bottom
- clipboard option allows args
- added all the available headers for `Item` and `ItemDetais` (everything in the 1password json response)

# 0.1.0

- navigate up and down with j,k (or Ctrl-U, Ctrl-D)
- press enter to see hidden info
- yank into clipboard with y
- sort by column
