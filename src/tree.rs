use crate::column::ColumnType;
use crate::column::{Cell, FileItem, FileItemPtr, GitStatus};
use crate::fs_utils;
use log::*;
use nvim_rs::{exttypes::Buffer, runtime::AsyncWrite, Neovim, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::From;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use tokio::fs;

#[derive(Clone)]
pub enum SplitType {
    Vertical,
    Horizontal,
    No,
    Tab,
    Floating,
}

impl Into<&'static str> for SplitType {
    fn into(self) -> &'static str {
        match self {
            SplitType::Vertical => "vertical",
            SplitType::Horizontal => "horizontal",
            SplitType::No => "no",
            SplitType::Tab => "tab",
            SplitType::Floating => "floating",
        }
    }
}

impl From<&str> for SplitType {
    fn from(s: &str) -> SplitType {
        match s {
            "vertical" => SplitType::Vertical,
            "horizontal" => SplitType::Horizontal,
            "no" => SplitType::No,
            "tab" => SplitType::Tab,
            "floating" => SplitType::Floating,
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
            split: SplitType::Vertical,
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
            let k_str = k.as_str();
            let k_str = &k_str[1..k.len() - 1];
            match k_str {
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
    fileitems: Vec<FileItemPtr>,
    expand_store: HashMap<String, bool>,
    // git_map: HashMap<String, GitStatus>,
    col_map: HashMap<ColumnType, Vec<Cell>>,
    targets: Vec<usize>,
}
impl Tree {
    pub async fn new<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        bufnr: (i8, Vec<u8>),
        buf: &Buffer<W>,
        nvim: &Neovim<W>,
        icon_ns_id: i64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        buf.set_option("ft", Value::from("tree")).await?;
        buf.set_option("modifiable", Value::from(false)).await?;
        nvim.command("lua require('tree')").await?;
        nvim.execute_lua("buf_attach(...)", vec![buf.get_value().clone()])
            .await?;
        Ok(Self {
            bufnr,
            icon_ns_id,
            config: Default::default(),
            fileitems: Default::default(),
            expand_store: Default::default(),
            // git_map: Default::default(),
            col_map: Default::default(),
            targets: Default::default(),
        })
    }
    pub fn get_fileitem(&self, idx: usize) -> &FileItem {
        &self.fileitems[idx]
    }
    pub async fn change_root<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        path_str: &str,
        nvim: &Neovim<W>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = std::path::Path::new(path_str);
        if !path.is_dir() {
            return Ok(());
        }
        let root_path = fs::canonicalize(path).await?;
        let root_path_str = root_path.to_str().unwrap();
        self.expand_store.insert(root_path_str.to_owned(), true);

        // TODO: update git map
        self.targets.clear();
        self.col_map.clear();

        let filemeta = fs::metadata(root_path_str).await?;
        let mut fileitems = vec![Arc::new(FileItem::new(root_path, filemeta))];
        self.entry_info_recursively(fileitems[0].clone(), &mut fileitems)
            .await?;
        self.fileitems = fileitems;

        // make line for each file item.
        // first the root cell
        self.insert_cells(0, true);
        let mut ret = Vec::new();
        ret.push(self.makeline(0));

        // then the cells below
        for pos in 1..self.fileitems.len() {
            self.insert_cells(pos, false);
            ret.push(self.makeline(pos));
        }

        self.buf_set_lines(nvim, 0, -1, true, ret).await?;
        Ok(())
    }

    // set the content of the buffer
    async fn buf_set_lines<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &self,
        nvim: &Neovim<W>,
        start: i64,
        end: i64,
        strict: bool,
        replacement: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buf = Buffer::new(Value::Ext(self.bufnr.0, self.bufnr.1.clone()), nvim.clone());
        buf.set_option("modifiable", Value::from(true)).await?;
        buf.set_lines(start, end, strict, replacement).await?;
        buf.set_option("modifiable", Value::from(false)).await?;
        Ok(())
    }

    // insert the cell at the given row
    fn insert_cells(&mut self, row_nr: usize, is_root: bool) {
        let fileitem = &self.fileitems[row_nr];
        let mut start = 0;
        let mut byte_start = 0;
        for col in &self.config.columns {
            let mut cell = Cell::new(self, fileitem, col.clone(), is_root);
            cell.byte_start = byte_start;
            cell.byte_end = byte_start + cell.text.len();
            cell.col_start = start;

            // TODO: count grid for file name
            cell.col_end = start + cell.text.len();
            // NOTE: alignment
            if *col == ColumnType::FILENAME {
                let stop = KSTOP - cell.col_end;
                if stop > 0 {
                    cell.col_end += KSTOP;
                    cell.byte_end += KSTOP;
                } else if is_root && KSTOP > cell.col_start + 5 {
                    // TODO: implement this
                }
            }
            let sep = if *col == ColumnType::INDENT { 0 } else { 1 };
            start = cell.col_end + sep;
            byte_start = cell.byte_end + sep;
            if !self.col_map.contains_key(col) {
                self.col_map.insert(col.clone(), Vec::new());
            }
            // TODO: inefficient here
            self.col_map.get_mut(col).unwrap().insert(row_nr, cell);
        }
    }

    fn entry_info_recursively<'a>(
        &'a self,
        item: Arc<FileItem>,
        fileitem_lst: &'a mut Vec<FileItemPtr>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + 'a + Send>> {
        Box::pin(async move {
            let mut read_dir = fs::read_dir(&item.path).await?;
            let mut entries = Vec::new();
            // filter: dirs, files, no dot and dot dot
            while let Some(entry) = read_dir.next_entry().await? {
                // skip hidden file or dot or dot dot
                if fs_utils::is_dot_or_dotdot(&entry)
                    || (!self.config.show_ignored_files && fs_utils::is_hidden(&entry))
                {
                    continue;
                }
                let metadata = entry.metadata().await?;
                let file_type = entry.file_type().await?;
                entries.push((entry, metadata, file_type));
            }
            if entries.len() <= 0 {
                return Ok(());
            }
            // directory first, name order
            entries.sort_by(|l, r| {
                if l.1.is_dir() && !r.1.is_dir() {
                    Ordering::Less
                } else if !l.1.is_dir() && l.1.is_dir() {
                    Ordering::Greater
                } else {
                    l.0.file_name().cmp(&r.0.file_name())
                }
            });
            let level = item.level + 1;
            let mut i = 0;
            let count = entries.len();
            for entry in entries {
                let mut fileitem = FileItem::new(fs::canonicalize(entry.0.path()).await?, entry.1);
                fileitem.level = level;
                fileitem.parent = Some(item.clone());
                if i == count - 1 {
                    fileitem.last = true;
                }
                i += 1;
                if let Some(expand) = self.expand_store.get(fileitem.path.to_str().unwrap()) {
                    if *expand {
                        fileitem.opened_tree = true;
                        fileitem_lst.push(Arc::new(fileitem));
                        self.entry_info_recursively(item.clone(), fileitem_lst)
                            .await?;
                    } else {
                        fileitem_lst.push(Arc::new(fileitem));
                    }
                } else {
                    fileitem_lst.push(Arc::new(fileitem));
                }
            }
            Ok(())
        })
    }

    fn makeline(&self, pos: usize) -> String {
        let mut start = 0;
        let mut line = String::new();
        for col in &self.config.columns {
            let cell = &self.col_map[col][pos];
            unsafe {
                line.push_str(&String::from_utf8_unchecked(vec![
                    b' ';
                    cell.col_start - start
                ]));
            }
            line.push_str(&cell.text);
            let len = cell.byte_end - cell.byte_start - cell.text.len();
            let space_after = unsafe { String::from_utf8_unchecked(vec![b' '; len]) };
            line.push_str(&space_after);
            start = cell.col_end;
        }
        line
    }
}
