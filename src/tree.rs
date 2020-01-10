use crate::column::ColumnType;
use crate::column::{Cell, FileItem, GitStatus};
use log::*;
use nvim_rs::Value;
use std::collections::HashMap;
use std::convert::From;
use std::io;
use tokio::fs;

#[derive(Clone)]
pub enum SplitType {
    Vertical,
    Horizontal,
    No,
    Tab,
    Floating,
}

impl Into<u8> for SplitType {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<&str> for SplitType {
    fn from(s: &str) -> SplitType {
        match s {
            "vertical" => SplitType::Vertical,
            "horizontal" => SplitType::Horizontal,
            "no" => SplitType::No,
            "Tab" => SplitType::Tab,
            "Floating" => SplitType::Floating,
            _ => SplitType::Vertical,
        }
    }
}

// State parameters for Tree
pub struct Config {
    pub auto_cd: bool,
    pub auto_recursive_level: u16,
    pub columns: Vec<ColumnType>,
    pub ignored_files: String,
    pub show_ignored_files: bool,
    pub profile: bool,
    pub root_marker: String,

    pub search: String,
    pub session_file: String,
    pub sort: String,

    pub listed: bool,
    pub buffer_name: String,

    pub direction: String,
    pub split: SplitType,
    pub winrelative: String,
    pub winheight: u16,
    pub winwidth: u16,
    pub wincol: u16,
    pub winrow: u16,
    pub new: bool,
    pub toggle: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_cd: false,
            auto_recursive_level: 0,
            columns: vec![
                ColumnType::MARK,
                ColumnType::INDENT,
                ColumnType::GIT,
                ColumnType::ICON,
                ColumnType::FILENAME,
                ColumnType::SIZE,
                ColumnType::TIME,
            ],
            ignored_files: String::new(),
            show_ignored_files: false,
            profile: false,
            root_marker: "[in]: ".to_owned(),
            search: String::new(),
            session_file: String::new(),
            sort: String::new(),

            listed: false,
            buffer_name: String::from("string"),

            direction: String::new(),
            split: SplitType::No,
            winrelative: String::from("editor"),
            winheight: 30,
            winwidth: 50,
            wincol: 0,
            winrow: 0,
            new: false,
            toggle: false,
        }
    }
}

impl Config {
    pub fn update(&mut self, cfg: &HashMap<String, Value>) {
        // TODO: handle type mismatch
        for (k, v) in cfg {
            match k.as_str() {
                "auto_recursive_level" => {
                    if let Some(v) = v.as_u64() {
                        self.auto_recursive_level = v as u16
                    } else {
                        warn!("type mismatch for auto_recursive_level: {}", v)
                    }
                }

                "wincol" => self.wincol = v.as_u64().unwrap() as u16,
                "winheigth" => self.winheight = v.as_u64().unwrap() as u16,
                "winrow" => self.winrow = v.as_u64().unwrap() as u16,
                "winwidth" => self.winwidth = v.as_u64().unwrap() as u16,
                "auto_cd" => self.auto_cd = v.as_bool().unwrap(),
                "listed" => self.listed = v.as_bool().unwrap(),
                "new" => self.new = v.as_bool().unwrap(),
                "profile" => self.profile = v.as_bool().unwrap(),
                "show_ignored_files" => self.show_ignored_files = v.as_bool().unwrap(),
                "toggle" => self.toggle = v.as_bool().unwrap(),
                "root_marker" => self.root_marker = v.as_str().unwrap().to_owned(),
                "buffer_name" => self.buffer_name = v.as_str().unwrap().to_owned(),
                "direction" => self.direction = v.as_str().unwrap().to_owned(),
                "ignored_files" => self.ignored_files = v.as_str().unwrap().to_owned(),
                "search" => self.search = v.as_str().unwrap().to_owned(),
                "session_file" => self.session_file = v.as_str().unwrap().to_owned(),
                "sort" => self.sort = v.as_str().unwrap().to_owned(),
                "winrelative" => self.winrelative = v.as_str().unwrap().to_owned(),
                "split" => self.split = SplitType::from(v.as_str().unwrap()),
                "columns" => {
                    self.columns.clear();
                    for col in v.as_str().unwrap().split(":") {
                        self.columns.push(ColumnType::from(col));
                    }
                }
                _ => error!("Unsupported member: {}", k),
            };
        }
    }
}

const KSTOP: usize = 90;

pub struct Tree {
    pub bufnr: (i8, Vec<u8>), // use bufnr to avoid tedious generic code
    pub icon_ns_id: i64,
    pub config: Config,
    fileitems: Vec<FileItem>,
    expand_store: HashMap<String, bool>,
    git_map: HashMap<String, GitStatus>,
    col_map: HashMap<ColumnType, Vec<Cell>>,
}
impl Tree {
    pub fn new(bufnr: (i8, Vec<u8>), icon_ns_id: i64) -> Self {
        Self {
            bufnr,
            icon_ns_id,
            config: Default::default(),
            fileitems: Default::default(),
            expand_store: Default::default(),
            git_map: Default::default(),
            col_map: Default::default(),
        }
    }
    pub fn get_fileitem(&self, idx: usize) -> &FileItem {
        &self.fileitems[idx]
    }
    pub async fn change_root(&mut self, path_str: &str) -> io::Result<()> {
        let path = std::path::Path::new(path_str);
        if !path.is_dir() {
            return Ok(());
        }
        let root_path = fs::canonicalize(path).await?;
        let root_path_str = root_path.to_str().unwrap();
        self.expand_store.insert(root_path_str.to_owned(), true);
        let filemeta = fs::metadata(root_path_str).await?;
        self.fileitems.push(FileItem::new(root_path, filemeta));
        self.insert_root_cell(0);
        Ok(())
    }

    fn insert_root_cell(&mut self, idx: usize) {
        let ft = &self.fileitems[idx];
        let mut start = 0;
        let mut byte_start = 0;
        for col in &self.config.columns {
            let mut cell = Cell::new(self, ft, col.clone());
            cell.col_start = start;
            cell.byte_start = byte_start;

            // speical for root cell
            if *col == ColumnType::FILENAME {
                let mut text = self.config.root_marker.clone();
                text.push_str(ft.path.to_str().unwrap());
                cell.text = text;
            }

            // char size is not always 1, TODO: count grid
            cell.byte_end = byte_start + cell.text.len();
            cell.col_end = start + cell.text.len();

            // NOTE: alignment
            if *col == ColumnType::FILENAME {
                let stop = KSTOP - cell.col_end;
                if stop > 0 {
                    cell.col_end += KSTOP;
                    cell.byte_end += KSTOP;
                }
            }

            let sep = if *col == ColumnType::INDENT { 0 } else { 1 };
            start = cell.col_end + sep;
            byte_start = cell.byte_end + sep;
            if !self.col_map.contains_key(col) {
                self.col_map.insert(col.clone(), Vec::new());
            }
            // TODO: inefficient here
            self.col_map.get_mut(col).unwrap().insert(idx, cell);
        }
    }
}
