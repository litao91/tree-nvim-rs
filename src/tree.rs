use crate::tree_handler::TreeHandler;
use nvim_rs::{create, exttypes::Buffer, runtime::Command, Handler, Neovim, Value};
pub struct Tree {
    bufnr: (i8, Vec<u8>), // use bufnr to avoid tedious generic code
    icon_ns_id: i64
}

impl Tree {
    pub fn new(bufnr: (i8, Vec<u8>), icon_ns_id: i64) -> Self {
        Self {
            bufnr,
            icon_ns_id
        }
    }
}
