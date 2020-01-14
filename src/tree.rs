use crate::column::ColumnType;
use crate::column::{Cell, FileItem, FileItemPtr, GitStatus};
use crate::errors::ArgError;
use crate::fs_utils;
use log::*;
use nvim_rs::{
    exttypes::{Buffer, Window},
    runtime::AsyncWrite,
    Neovim, Value,
};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::From;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::fs;

#[derive(Default, Debug, Clone)]
pub struct Context {
    pub cursor: u64,
    pub drives: Vec<String>,
    pub prev_winid: u64,
    pub visual_start: u64,
    pub visual_end: u64,
    pub prev_bufnr: Option<(i8, Vec<u8>)>,
}

impl Context {
    pub fn update(&mut self, key: &str, val: Value) {
        match key {
            "prev_bufnr" => match val {
                Value::Integer(v) => {
                    self.prev_bufnr = Some((0, vec![v.as_u64().unwrap() as u8]));
                }
                Value::Ext(v1, v2) => self.prev_bufnr = Some((v1, v2)),
                _ => {
                    error!("Unknown value: {}", val);
                }
            },
            "cursor" => match val {
                Value::Integer(v) => {
                    self.cursor = if let Some(v) = v.as_u64() {
                        v
                    } else {
                        error!("Can't convert value {} to u64", val);
                        return;
                    }
                }
                _ => {
                    error!("Unknown value: {}", val);
                }
            },
            "prev_winid" => match val {
                Value::Integer(v) => {
                    self.prev_winid = if let Some(v) = v.as_u64() {
                        v
                    } else {
                        error!("Can't convert value {} to u64", val);
                        return;
                    }
                }
                _ => {
                    error!("Unknown value: {}", val);
                }
            },
            "visual_start" => match val {
                Value::Integer(v) => {
                    self.visual_start = if let Some(v) = v.as_u64() {
                        v
                    } else {
                        error!("Can't convert value {} to u64", val);
                        return;
                    }
                }
                _ => {
                    error!("Unknown value: {}", val);
                }
            },
            "visual_end" => match val {
                Value::Integer(v) => {
                    self.visual_end = if let Some(v) = v.as_u64() {
                        v
                    } else {
                        error!("Can't convert value {} to u64", val);
                        return;
                    }
                }
                _ => {
                    error!("Unknown value: {}", val);
                }
            },
            _ => {
                warn!("Unsupported member: {}", key);
            }
        }
    }
}

#[derive(Clone, Debug)]
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
#[derive(Debug)]
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

fn val_to_u16(v: &Value) -> Result<u16, Box<dyn std::error::Error>> {
    if let Some(v_str) = v.as_str() {
        Ok(v_str.parse::<u16>()?)
    } else {
        match v.as_u64() {
            Some(v) => Ok(v as u16),
            None => Err(Box::new(crate::errors::ArgError::new("Type mismatch"))),
        }
    }
}

fn val_to_string(v: &Value) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(v_str) = v.as_str() {
        Ok(v_str.to_owned())
    } else {
        Err(Box::new(crate::errors::ArgError::new("Type mismatch")))
    }
}

fn val_to_bool(v: &Value) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(v_str) = v.as_str() {
        Ok(v_str.parse::<bool>()?)
    } else {
        match v.as_bool() {
            Some(v) => Ok(v),
            None => Err(Box::new(crate::errors::ArgError::new("Type mismatch"))),
        }
    }
}

impl Config {
    pub fn update(
        &mut self,
        cfg: &HashMap<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: handle type mismatch
        for (k, v) in cfg {
            info!("k: {:?}, v: {:?}", k, v);
            match k.as_str() {
                "auto_recursive_level" => self.auto_recursive_level = val_to_u16(v)?,
                "wincol" => self.wincol = val_to_u16(v)?,
                "winheigth" => self.winheight = val_to_u16(v)?,
                "winrow" => self.winrow = val_to_u16(v)?,
                "winwidth" => self.winwidth = val_to_u16(v)?,
                "auto_cd" => self.auto_cd = val_to_bool(v)?,
                "listed" => self.listed = val_to_bool(v)?,
                "new" => self.new = val_to_bool(v)?,
                "profile" => self.profile = val_to_bool(v)?,
                "show_ignored_files" => self.show_ignored_files = val_to_bool(v)?,
                "toggle" => self.toggle = val_to_bool(v)?,
                "root_marker" => self.root_marker = val_to_string(v)?,
                "buffer_name" => self.buffer_name = val_to_string(v)?,
                "direction" => self.direction = val_to_string(v)?,
                "ignored_files" => self.ignored_files = val_to_string(v)?,
                "search" => self.search = val_to_string(v)?,
                "session_file" => self.session_file = val_to_string(v)?,
                "sort" => self.sort = val_to_string(v)?,
                "winrelative" => self.winrelative = val_to_string(v)?,
                "split" => {
                    self.split = SplitType::from(match v.as_str() {
                        Some(s) => s,
                        None => {
                            return Err(Box::new(crate::errors::ArgError::new("Str type expected")))
                        }
                    })
                }

                "columns" => {
                    self.columns.clear();
                    for col in match v.as_str() {
                        Some(v) => v.split(":"),
                        None => {
                            return Err(Box::new(crate::errors::ArgError::new("Str type expected")))
                        }
                    } {
                        // info!("col:{}", col);
                        self.columns.push(ColumnType::from(col));
                    }
                }
                _ => error!("Unsupported member: {}", k),
            };
        }
        Ok(())
    }
}

const KSTOP: usize = 90;

#[derive(Debug)]
pub struct Tree {
    pub bufnr: (i8, Vec<u8>), // use bufnr to avoid tedious generic code
    pub icon_ns_id: i64,
    pub config: Config,
    fileitems: Vec<FileItemPtr>,
    expand_store: HashMap<String, bool>,
    // git_map: HashMap<String, GitStatus>,
    col_map: HashMap<ColumnType, Vec<Cell>>,
    targets: Vec<usize>,
    cursor_history: HashMap<String, i64>,
    ctx: Context,
}
impl Tree {
    pub fn is_item_opened(&self, path: &str) -> bool {
        match self.expand_store.get(path) {
            Some(v) => *v,
            None => false,
        }
    }
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
            cursor_history: Default::default(),
            ctx: Default::default(),
        })
    }
    pub async fn action<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        action: &str,
        args: Value,
        ctx: Context,
    ) {
        info!(
            "Action: {:?}, \n args: {:?}, \n ctx: {:?}",
            action, args, ctx
        );
        self.ctx = ctx.clone();
        match action {
            "drop" => self.action_drop(nvim, args).await,
            "open_tree" | "open_directory" => self.action_open_directory(nvim, args).await,
            _ => error!("Unknown action: {}", action),
        }
    }

    pub async fn action_drop<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        args: Value,
    ) {
        let info: String;
        let should_change_root;
        if let Some(cur) = self.fileitems.get(self.ctx.cursor as usize - 1) {
            info = cur.path.to_str().unwrap().to_owned();
            if cur.metadata.is_dir() {
                should_change_root = true;
            } else {
                should_change_root = false;
            }
        } else {
            error!("drop: invalid cursor position");
            return;
        }
        if should_change_root {
            match self.change_root(&info, nvim).await {
                Ok(_) => {}
                Err(e) => error!("Error changing root: {:?}", e),
            }
        } else {
            match nvim
                .execute_lua("drop(...)", vec![args, Value::from(info)])
                .await
            {
                Ok(_) => {}
                Err(e) => error!("Error: {:?}", e),
            }
        }
    }

    pub async fn action_open_directory<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _args: Value,
    ) {
        let idx = self.ctx.cursor as usize - 1;
        let cur = match self.fileitems.get(idx) {
            Some(fi) => fi,
            None => {
                error!("Index out of bound: {}", idx);
                return;
            }
        }
        .clone();
        let root_path = match cur.path.to_str() {
            Some(path) => path,
            None => {
                error!("filename error");
                return;
            }
        };
        let is_opened = match self.expand_store.get(root_path) {
            Some(v) => *v,
            None => false,
        };
        if cur.metadata.is_dir() && !is_opened {
            let mut child_fileitem = Vec::new();
            match self
                .entry_info_recursively(cur.clone(), &mut child_fileitem, idx + 1)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!("Err: {:?}", e);
                    return;
                }
            }
            self.expand_store.insert(root_path.to_owned(), true);
            // icon should be open
            self.update_cells(idx, idx + 1);
            let child_item_size = child_fileitem.len();
            match self.insert_items_and_cells(idx + 1, child_fileitem) {
                Ok(_) => {}
                Err(e) => error!("Err: {:?}", e),
            };
            // update lines
            let end = idx + child_item_size + 1;
            let ret = (idx..end).map(|i| self.makeline(i)).collect();
            match self
                .buf_set_lines(nvim, idx as i64, (idx + 1) as i64, true, ret)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!("{:?}", e);
                    return;
                }
            }
            match self.hl_lines(&nvim, idx, idx + 1 + child_item_size).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{:?}", e);
                    return;
                }
            }
        }
    }

    pub fn update_cells(&mut self, sl: usize, el: usize) {
        let cells = self.make_cells(&self.fileitems[sl..el], sl == 0);
        for (col, cells) in cells {
            if !self.col_map.contains_key(&col) {
                self.col_map.insert(col.clone(), Vec::new());
            }
            self.col_map.get_mut(&col).unwrap().splice(sl..el, cells);
        }
    }

    pub fn get_context_value(&self, cursor: usize) -> Value {
        let idx = cursor - 1;
        let ft = self.fileitems.get(idx).unwrap();
        info!("get context of: {:?}", ft.path);
        Value::Map(vec![
            (Value::from("is_directory"), Value::from(ft.metadata.is_dir())),
            (Value::from("is_opened_tree"), Value::from(self.is_item_opened(ft.path.to_str().unwrap()))),
            (Value::from("level"), Value::from(ft.level))
        ])
    }


    pub fn get_fileitem(&self, idx: usize) -> &FileItem {
        &self.fileitems[idx]
    }
    // TODO: use unsafe or mutex?
    pub unsafe fn get_fileitem_mut(&self, idx: usize) -> Option<&mut FileItem> {
        Some(&mut *(self.fileitems.get(idx)?.as_ref() as *const FileItem as *mut FileItem))
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
        let root_path_str = if let Some(p) = root_path.to_str() {
            p
        } else {
            return Err(Box::new(ArgError::from_string(format!(
                "Invalid path {:?}",
                root_path
            ))));
        };
        let last_cursor = match self.cursor_history.get(root_path_str) {
            Some(v) => Some(*v),
            None => None,
        };
        self.expand_store.insert(root_path_str.to_owned(), true);

        // TODO: update git map
        self.targets.clear();
        self.col_map.clear();
        self.fileitems.clear();

        let filemeta = fs::metadata(root_path_str).await?;
        let mut fileitems = vec![Arc::new(FileItem::new(root_path, filemeta, 0))];
        self.entry_info_recursively(fileitems[0].clone(), &mut fileitems, 1)
            .await?;
        self.insert_items_and_cells(0, fileitems)?;

        let ret = (0..self.fileitems.len())
            .map(|i| self.makeline(i))
            .collect();
        self.buf_set_lines(nvim, 0, -1, true, ret).await?;
        if let Some(v) = last_cursor {
            let win = Window::new(Value::from(0), nvim.clone());
            win.set_cursor((0, v)).await?;
        }
        self.hl_lines(&nvim, 0, self.fileitems.len()).await?;
        Ok(())
    }

    fn make_cells(
        &self,
        items: &[FileItemPtr],
        first_item_is_root: bool,
    ) -> Vec<(ColumnType, Vec<Cell>)> {
        let mut r = Vec::new();
        for col in &self.config.columns {
            r.push((col.clone(), Vec::new()))
        }
        let mut is_first = true;
        for fileitem in items {
            let mut start = 0;
            let mut byte_start = 0;
            let is_root = first_item_is_root && is_first;
            for i in 0..self.config.columns.len() {
                let col = &self.config.columns[i];
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
                r[i].1.push(cell);
            }
            is_first = false;
        }
        r
    }

    // insert at the pos
    fn insert_items_and_cells(
        &mut self,
        pos: usize,
        items: Vec<FileItemPtr>,
    ) -> Result<(), ArgError> {
        if pos > self.fileitems.len() {
            return Err(ArgError::new("pos larger than the fileitem size"));
        }
        let is_first_item_root = pos == 0;
        // make cells
        let cells = self.make_cells(&items, is_first_item_root);
        // insert items
        let size_to_insert = items.len();
        self.fileitems.splice(pos..pos, items);
        // update the indices
        if pos + size_to_insert < self.fileitems.len() {
            for i in pos + size_to_insert..self.fileitems.len() {
                // TODO: is it safe here?
                // NOTE: this should be safe
                // 1. this is the only place modifying the index
                // 2. the data is in TreeHandler::data, which is protected by a mutex => impossible
                //    to be modified concurrently.
                unsafe {
                    (&mut *(self.fileitems[i].as_ref() as *const FileItem as *mut FileItem)).id = i;
                }
            }
        }
        // insert the cells
        for (col, cells) in cells {
            if !self.col_map.contains_key(&col) {
                self.col_map.insert(col.clone(), Vec::new());
            }
            self.col_map.get_mut(&col).unwrap().splice(pos..pos, cells);
        }
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
        mut start_id: usize,
    ) -> Pin<Box<dyn Future<Output = Result<usize, Box<dyn std::error::Error>>> + 'a + Send>> {
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
                return Ok(start_id);
            }
            // directory first, name order
            entries.sort_by(|l, r| {
                if l.1.is_dir() && !r.1.is_dir() {
                    Ordering::Less
                } else if !l.1.is_dir() && r.1.is_dir() {
                    Ordering::Greater
                } else {
                    l.0.file_name().cmp(&r.0.file_name())
                }
            });
            let level = item.level + 1;
            let mut i = 0;
            let count = entries.len();
            for entry in entries {
                let mut fileitem =
                    FileItem::new(fs::canonicalize(entry.0.path()).await?, entry.1, start_id);
                start_id += 1;
                fileitem.level = level;
                fileitem.parent = Some(item.clone());
                if i == count - 1 {
                    fileitem.last = true;
                }
                i += 1;
                if let Some(expand) = self.expand_store.get(fileitem.path.to_str().unwrap()) {
                    if *expand {
                        fileitem_lst.push(Arc::new(fileitem));
                        start_id = self
                            .entry_info_recursively(item.clone(), fileitem_lst, start_id)
                            .await?;
                    } else {
                        fileitem_lst.push(Arc::new(fileitem));
                    }
                } else {
                    fileitem_lst.push(Arc::new(fileitem));
                }
            }
            Ok(start_id)
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

    // [sl, el)
    async fn hl_lines<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &self,
        nvim: &Neovim<W>,
        sl: usize,
        el: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buf = Buffer::new(Value::Ext(self.bufnr.0, self.bufnr.1.clone()), nvim.clone());
        for i in sl..el {
            let fileitem = &self.fileitems[i];
            for col in &self.config.columns {
                let cell = &self.col_map.get(col).unwrap()[i];
                if let Some(hl_group) = &cell.hl_group {
                    buf.add_highlight(
                        self.icon_ns_id,
                        hl_group,
                        i as i64,
                        cell.byte_start as i64,
                        (cell.byte_start + cell.text.len()) as i64,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }
}
