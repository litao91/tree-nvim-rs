# tree-nvim-rs - File explorer for nvim powered by rust

Inspired by [tree.nvim](https://github.com/zgpio/tree.nvim) and Defx, in fact the nvim/lua part is almost identical to that of `tree.nvim` 

## Features

* The performance is comparable to the C++ version (`tree.nvim`)
* The binary size is much smaller than `tree.nvim` , due to the QT dependency, the binary of `tree.nvim` is added up to more than 30M

## TODO
- [x] create tree
- [x] open/close, open or close
- [x] new_file
- [x] rename
- [ ] delete
- [ ] yank_path
- [ ] open/close recursively
- [ ] selection
- [ ] resize