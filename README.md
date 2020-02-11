# tree-nvim-rs - File explorer for nvim powered by rust

Inspired by [tree.nvim](https://github.com/zgpio/tree.nvim) and Defx, in fact the nvim/lua part is almost identical to that of `tree.nvim` 

## Features

* The performance is comparable to the C++ version (`tree.nvim`)
* The binary size is much smaller than `tree.nvim` , due to the QT dependency, the binary of `tree.nvim` is added up to more than 30M

## TODO

- [x] cd
- [ ] copy
- [x] drop
- [ ] move
- [ ] open
- [ ] multi
- [ ] remove_trash
- [x] create tree
- [x] open/close, open or close
- [x] new_file
- [x] rename
- [x] delete
- [x] toggle hidden files
- [x] toggle select
- [x] toggle select all
- [x] clear select all
- [x] yank_path
- [ ] open/close recursively
- [x] selection
- [x] resize
- [x] git status
- [x] update git map && load git status on git command
- [ ] search
- [x] redraw
- [ ] test cases
- [x] Custom
- [x] size and time column
- [x] better file name alignment
- [x] Space cell
- [ ] More file types recognization and icon customization
