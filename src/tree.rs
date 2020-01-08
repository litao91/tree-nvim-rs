use crate::tree_handler::TreeHandler;
use nvim_rs::{create, exttypes::Buffer, runtime::Command, Handler, Neovim, Value};
pub struct Tree {
    bufnr: Value,
}
