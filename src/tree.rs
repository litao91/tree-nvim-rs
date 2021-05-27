use crate::column::ColumnType;
use crate::column::{ColumnCell, FileItem, FileItemPtr};
use crate::errors::ArgError;
use async_std::sync::{Arc, Mutex, RwLock};
use fs_extra;
use futures::io::AsyncWrite;
use git2::{Repository, Status};
use log::*;
use nvim_rs::{
    exttypes::{Buffer, Window},
    Neovim, Value,
};
use path_clean::PathClean;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::From;
use std::env;
use std::fmt;
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use unicode_width::UnicodeWidthStr;

pub fn absolute_path<P>(path: P) -> io::Result<PathBuf>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.is_absolute() {
        Ok(path.to_path_buf().clean())
    } else {
        Ok(env::current_dir()?.join(path).clean())
    }
}

#[derive(Default, Debug, Clone)]
pub struct Context {
    pub cursor: u64,
    pub drives: Vec<String>,
    pub visual_start: u64,
    pub visual_end: u64,
    pub prev_bufnr: Option<Value>,
}

impl Context {
    pub fn update(&mut self, key: &str, val: Value) {
        match key {
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
                warn!("Context: Unsupported member: {}", key);
            }
        }
    }
}

pub enum ClipboardMode {
    COPY,
    MOVE,
}

static CLIPBOARD_MODE: RwLock<ClipboardMode> = RwLock::new(ClipboardMode::COPY);
static CLIPBOARD: RwLock<Vec<std::path::PathBuf>> = RwLock::new(Vec::new());

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
        }
    }
}

fn val_to_u16(v: &Value) -> Result<u16, Box<dyn std::error::Error>> {
    if let Some(v_str) = v.as_str() {
        Ok(v_str.parse::<u16>()?)
    } else {
        match v.as_u64() {
            Some(v) => Ok(v as u16),
            None => Err(Box::new(crate::errors::ArgError::new(
                "Type mismatch: str u64",
            ))),
        }
    }
}

fn val_to_string(v: &Value) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(v_str) = v.as_str() {
        Ok(v_str.to_owned())
    } else {
        Err(Box::new(crate::errors::ArgError::new(
            "Type mismatch: str expected",
        )))
    }
}

fn val_to_bool(v: &Value) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(v_str) = v.as_str() {
        Ok(v_str.parse::<bool>()?)
    } else {
        match v.as_bool() {
            Some(v) => Ok(v),
            None => match v.as_i64() {
                Some(vi) => Ok(vi == 1),
                None => Err(Box::new(crate::errors::ArgError::from_string(format!(
                    "Type mismatch: bool expected, but {:?} found",
                    v
                )))),
            },
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
                "auto_cd" => {
                    self.auto_cd = val_to_bool(v).map_err(|e| {
                        ArgError::from_string(format!("auto_cd need boolean type: {:?}", e))
                    })?
                }
                "listed" => {
                    self.listed = val_to_bool(v).map_err(|e| {
                        ArgError::from_string(format!("Config: auto_cd need boolean type: {:?}", e))
                    })?
                }
                "profile" => {
                    self.profile = val_to_bool(v).map_err(|e| {
                        ArgError::from_string(format!("profile need boolean type: {:?}", e))
                    })?
                }
                "show_ignored_files" => {
                    self.show_ignored_files = val_to_bool(v).map_err(|e| {
                        ArgError::from_string(format!(
                            "show_ignored_files need boolean type: {:?}",
                            e
                        ))
                    })?
                }
                "root_marker" => self.root_marker = val_to_string(v)?,
                "ignored_files" => self.ignored_files = val_to_string(v)?,
                "search" => self.search = val_to_string(v)?,
                "session_file" => self.session_file = val_to_string(v)?,
                "sort" => self.sort = val_to_string(v)?,
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
                _ => warn!("Config: Unsupported member: {}", k),
            };
        }
        Ok(())
    }
}

const KSTOP: usize = 60;

pub struct Tree {
    pub bufnr: Value, // use bufnr to avoid tedious generic code
    pub icon_ns_id: i64,
    pub config: Config,
    selected_items: HashSet<usize>,
    file_items: Vec<FileItemPtr>,
    expand_store: HashMap<String, bool>,
    col_map: HashMap<ColumnType, Vec<ColumnCell>>,
    targets: Vec<usize>,
    cursor_history: HashMap<String, u64>,
    git_repo: Option<Mutex<Repository>>,
    pub git_map: HashMap<String, Status>,
}

impl Debug for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tree {{ bufnr: {:?}, icon_ns_id: {:?}, config: {:?} }}",
            self.bufnr, self.icon_ns_id, self.config
        )
    }
}

impl Tree {
    pub async fn new<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        bufnr: Value,
        buf: &Buffer<W>,
        nvim: &Neovim<W>,
        icon_ns_id: i64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        buf.set_option("ft", Value::from("tree")).await?;
        buf.set_option("modifiable", Value::from(false)).await?;
        nvim.command("lua tree = require('tree')").await?;
        nvim.execute_lua("tree.buf_attach(...)", vec![buf.get_value().clone()])
            .await?;
        Ok(Self {
            bufnr,
            icon_ns_id,
            config: Default::default(),
            file_items: Default::default(),
            expand_store: Default::default(),
            col_map: Default::default(),
            targets: Default::default(),
            cursor_history: Default::default(),
            selected_items: Default::default(),
            git_repo: None,
            git_map: Default::default(),
        })
    }
    pub fn is_item_opened(&self, path: &str) -> bool {
        match self.expand_store.get(path) {
            Some(v) => *v,
            None => false,
        }
    }
    pub fn is_item_selected(&self, idx: usize) -> bool {
        self.selected_items.contains(&idx)
    }
    pub fn init_git_repo<P: AsRef<Path>>(&mut self, path: P) {
        match Repository::discover(path) {
            Ok(repo) => self.git_repo = Some(Mutex::new(repo)),
            Err(e) => {
                info!("Not a git repo: {:?}", e);
            }
        }
    }
    pub fn update_git_map(&mut self) {
        if self.git_repo.is_none() {
            self.init_git_repo(&self.file_items[0].path.clone())
        }
        if let Some(ref mutex) = self.git_repo {
            if let Some(ref repo) = mutex.try_lock() {
                self.git_map.clear();
                match repo.statuses(None) {
                    Ok(statuses) => {
                        let work_dir = repo.workdir().unwrap();
                        for status in statuses.iter() {
                            self.git_map.insert(
                                work_dir
                                    .join(status.path().unwrap())
                                    .to_str()
                                    .unwrap()
                                    .to_owned(),
                                status.status(),
                            );
                        }
                        info!("git_map: {:?}", self.git_map);
                    }
                    Err(e) => error!("Fail to get status: {:?}", e),
                }
            } else {
                info!("We failed the race!");
            }
        } else {
            info!("Git not enabled");
        }
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
        match match action {
            "drop" => self.action_drop(nvim, args, ctx).await,
            "open_tree" => self.action_open_tree(nvim, args, ctx).await,
            "close_tree" => self.action_close_tree(nvim, args, ctx).await,
            "open_or_close_tree" => self.action_open_or_close_tree(nvim, args, ctx).await,
            "open_directory" => self.action_open_directory(nvim, args, ctx).await,
            "cd" => self.action_cd(nvim, args, ctx).await,
            "call" => self.action_call(nvim, args, ctx).await,
            "new_file" => self.action_new_file(nvim, args, ctx).await,
            "rename" => self.action_rename(nvim, args, ctx).await,
            "toggle_select" => self.action_toggle_select(nvim, args, ctx).await,
            "remove" => self.action_remove(nvim, args, ctx).await,
            "toggle_ignored_files" => self.action_show_ignored(nvim, args, ctx).await,
            "yank_path" => self.action_yank_path(nvim, args, ctx).await,
            "clear_select_all" => self.action_clear_select_all(nvim, args, ctx).await,
            "toggle_select_all" => self.action_toggle_select_all(nvim, args, ctx).await,
            "redraw" => self.action_redraw(nvim, args, ctx).await,
            "resize" => self.action_resize(nvim, args, ctx).await,
            "update_git_map" => self.action_update_git_map(nvim, args, ctx).await,
            "copy" => self.action_copy(nvim, args, ctx).await,
            "move" => self.action_move(nvim, args, ctx).await,
            "paste" => self.action_paste(nvim, args, ctx).await,
            _ => {
                error!("Unknown action: {}", action);
                return;
            }
        } {
            Ok(_) => {}
            Err(e) => error!("err: {:?}", e),
        }
    }

    pub fn save_cursor(&mut self, ctx: &Context) {
        if let Some(item) = self.file_items.get(0) {
            if let Some(path) = item.path.to_str() {
                self.cursor_history.insert(path.to_owned(), ctx.cursor);
            }
        }
    }

    pub async fn cwd_input<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        nvim: &Neovim<W>,
        cwd: &str,
        prompt: &str,
        text: &str,
        completion: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let save_cwd = nvim.call_function("getcwd", vec![]).await?;
        info!("cwd: {:?}", save_cwd);
        nvim.call_function("tree#util#cd", vec![Value::from(cwd)])
            .await?;

        let filename = if let Value::String(v) = nvim
            .call_function(
                "tree#util#input",
                vec![
                    Value::from(prompt),
                    Value::from(text),
                    Value::from(completion),
                ],
            )
            .await?
        {
            v.into_str().unwrap()
        } else {
            return Err(Box::new(ArgError::new("Wrong return type")));
        };

        nvim.call_function("tree#util#cd", vec![save_cwd]).await?;

        Ok(filename)
    }

    pub async fn confirm<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        nvim: &Neovim<W>,
        question: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if let Value::Integer(v) = nvim
            .call_function(
                "tree#util#confirm",
                vec![
                    Value::from(question),
                    Value::from("&Yes\n&No\n&Cancel"),
                    Value::from(2),
                ],
            )
            .await?
        {
            Ok(v.as_i64().unwrap() == 1)
        } else {
            Err(Box::new(ArgError::new("Invalid return type")))
        }
    }

    pub async fn redraw_subtree<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        parent_idx: usize,
        force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cur = match self.file_items.get(parent_idx) {
            Some(c) => c,
            None => return Err(Box::new(ArgError::new("Invalid index"))),
        }
        .clone();

        let idx = cur.id;
        let base_level = cur.level;
        let start = cur.id + 1;
        let mut end = start;
        for fi in &self.file_items[start..] {
            if fi.level <= base_level {
                break;
            }
            end += 1;
        }

        info!("remove range [{}, {})", start, end);
        let new_end;
        if force {
            self.remove_items_and_cells(start, end)?;
            let mut child_items = Vec::new();
            self.entry_info_recursively_sync(cur.clone(), &mut child_items, idx + 1)?;
            let child_item_size = child_items.len();
            self.insert_items_and_cells(start, child_items)?;
            new_end = start + child_item_size;
        } else {
            let cells = self.make_cells(&self.file_items[start..end], start == 0);
            for (col, cells) in cells {
                if !self.col_map.contains_key(&col) {
                    self.col_map.insert(col.clone(), Vec::new());
                }
                self.col_map
                    .get_mut(&col)
                    .unwrap()
                    .splice(start..end, cells);
            }
            new_end = end;
        }
        // the new end after adding the new file
        info!("redraw range [{}, {})", start, new_end);
        // update lines (zero based)
        let ret = (start..new_end).map(|i| self.makeline(i)).collect();
        self.buf_set_lines(nvim, start as i64, end as i64, true, ret)
            .await?;
        self.hl_lines(&nvim, start, new_end).await?;
        Ok(())
    }

    pub async fn action_redraw<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        _ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.redraw_subtree(nvim, 0, true).await?;
        Ok(())
    }

    pub async fn action_resize<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        arg: Value,
        _ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut args = match arg {
            Value::Array(v) => v,
            _ => {
                Err(ArgError::new("Invalid arg type"))?;
                return Ok(());
            }
        };
        if args.is_empty() {
            return Ok(());
        }
        args.push(self.bufnr.clone());
        info!(" args for resize: {:?}", args);
        // nvim.execute_lua("tree.print_message(...)", vec![Value::from("hello".to_owned())]).await?;
        nvim.execute_lua("tree.resize(...)", args).await?;
        Ok(())
    }

    pub async fn action_yank_path<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let paths_str = if self.selected_items.is_empty() {
            self.file_items[ctx.cursor as usize - 1]
                .path
                .to_str()
                .unwrap()
                .to_owned()
        } else {
            self.selected_items
                .iter()
                .map(|x| self.file_items[*x].path.to_str().unwrap().to_owned())
                .collect::<Vec<String>>()
                .join("\n")
        };
        nvim.call_function(
            "setreg",
            vec![Value::from("+"), Value::from(paths_str.as_str())],
        )
        .await?;
        nvim.execute_lua("tree.print_message(...)", vec![Value::from(paths_str)])
            .await?;
        Ok(())
    }

    pub async fn action_show_ignored<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        _ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.config.show_ignored_files = !self.config.show_ignored_files;
        self.redraw_subtree(nvim, 0, true).await?;
        Ok(())
    }

    pub async fn action_remove<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let args = match arg {
            Value::Array(v) => v,
            _ => {
                Err(ArgError::new("Invalid arg type"))?;
                return Ok(());
            }
        };

        let force = match args.get(0) {
            Some(Value::String(v)) => v.as_str().unwrap() == "true",
            _ => false,
        };
        let targets: Vec<&FileItem> = if self.selected_items.is_empty() {
            vec![&self.file_items[ctx.cursor as usize - 1].as_ref()]
        } else {
            self.selected_items
                .iter()
                .map(|x| self.file_items[*x].as_ref())
                .collect()
        };
        if !force {
            let message = if targets.len() == 1 {
                format!(
                    "Are you sure you want to delete {}?",
                    targets[0].path.to_str().unwrap()
                )
            } else {
                format!("Are you sure you want to delete {} files?", targets.len())
            };
            if !Self::confirm(nvim, message).await? {
                info!("Remove cancelled");
                return Ok(());
            }
        }
        for target in targets {
            if target.metadata.is_dir() {
                std::fs::remove_dir_all(&target.path)?;
            } else {
                std::fs::remove_file(&target.path)?;
            }
        }
        // redraw the entire tree
        self.redraw_subtree(nvim, 0, true).await?;

        Ok(())
    }
    pub async fn action_toggle_select<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let idx = (ctx.cursor - 1) as usize;
        if self.selected_items.contains(&idx) {
            self.selected_items.remove(&idx);
        } else {
            self.selected_items.insert(idx);
        }

        // soft redraw a single line
        self.update_cells(idx, idx + 1);
        let ret = vec![self.makeline(idx)];
        self.buf_set_lines(nvim, idx as i64, idx as i64 + 1, true, ret)
            .await?;
        self.hl_lines(&nvim, idx, idx + 1).await?;

        Ok(())
    }

    pub async fn action_clear_select_all<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        _ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.selected_items.clear();
        self.redraw_subtree(nvim, 0, false).await?;
        Ok(())
    }

    pub async fn action_toggle_select_all<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        _ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..self.file_items.len() {
            if !self.selected_items.remove(&i) {
                self.selected_items.insert(i);
            }
        }
        self.redraw_subtree(nvim, 0, false).await?;
        Ok(())
    }

    pub async fn action_rename<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("{:?}", _arg);
        let idx = ctx.cursor as usize - 1;
        let cur = &self.file_items[idx];
        let old_path = cur.path.to_str().unwrap();
        let cwd = self.file_items[0].path.to_str().unwrap();
        let msg = format!("New name: {} -> ", old_path);
        let new_filename = Self::cwd_input(nvim, cwd, &msg, old_path, "file").await?;
        if new_filename.is_empty() {
            return Ok(());
        }
        // let new_path = fs::canonicalize(cur.path.join(new_filename)).await?;
        let new_path = cur.path.join(new_filename);
        if new_path == cur.path {
            return Ok(());
        }
        info!("New path: {:?}", new_path);

        if new_path.exists() {
            let message = Value::from(format!("{} already exists", new_path.to_str().unwrap()));
            nvim.execute_lua("tree.print_message(...)", vec![message])
                .await?;
            return Err(Box::new(ArgError::new("File exists!")));
        }
        std::fs::rename(&cur.path, new_path)?;
        // TODO: no need to redraw the entire tree, we can redraw the parent and the target's
        // parent
        self.redraw_subtree(nvim, 0, true).await?;

        Ok(())
    }

    pub async fn action_new_file<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let idx = ctx.cursor as usize - 1;
        let cur = &self.file_items[idx];
        let cur_path_str = cur.path.to_str().unwrap();
        let idx_to_redraw;
        // idx == 0 => is_root
        let cwd = if self.is_item_opened(cur_path_str) || idx == 0 {
            idx_to_redraw = idx;
            cur_path_str
        } else if let Some(p) = cur.parent.as_ref() {
            idx_to_redraw = p.id;
            p.path.to_str().unwrap()
        } else {
            return Err(Box::new(ArgError::new(
                "can't find correct position to create new file",
            )));
        };
        let new_filename =
            Self::cwd_input(nvim, &cwd, "Please input a new filename: ", "", "file").await?;
        let is_dir = new_filename.ends_with('/');
        let mut filename = std::path::PathBuf::from(cwd);
        filename.push(new_filename);
        info!("New file name: {:?}", filename);
        let message = Value::from(format!("{} already exists", filename.to_str().unwrap()));
        if filename.exists() {
            nvim.execute_lua("tree.print_message(...)", vec![message])
                .await?;
            return Err(Box::new(ArgError::new("File exists!")));
        }
        if is_dir {
            std::fs::create_dir(filename)?;
        } else {
            let mut parent = filename.clone();
            parent.pop();
            std::fs::create_dir_all(parent)?;
            std::fs::File::create(filename)?;
        }

        self.redraw_subtree(nvim, idx_to_redraw, true).await?;

        Ok(())
    }
    pub async fn action_call<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let args = match arg {
            Value::Array(v) => v,
            _ => {
                Err(ArgError::new("Invalid arg type"))?;
                return Ok(());
            }
        };
        let func = if let Some(Value::String(v)) = args.get(0) {
            v.as_str().unwrap()
        } else {
            return Err(Box::new(ArgError::new("func not defined")));
        };
        let cur = &self.file_items[ctx.cursor as usize - 1];

        let ctx = Value::Map(vec![(
            Value::from("targets"),
            Value::Array(vec![Value::from(cur.path.to_str().unwrap())]),
        )]);
        nvim.call_function(func, vec![ctx]).await?;
        Ok(())
    }

    pub async fn action_cd<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.save_cursor(&ctx);
        let args = match arg {
            Value::Array(v) => v,
            _ => {
                Err(ArgError::new("Invalid arg type"))?;
                return Ok(());
            }
        };
        if !args.is_empty() {
            let dir = if let Some(d) = args[0].as_str() {
                d
            } else {
                Err(ArgError::new("Dir should be of type String"))?;
                return Ok(());
            };
            if dir == ".." {
                match self.file_items[0].path.clone().parent() {
                    Some(p) => self.change_root(p.to_str().unwrap(), nvim).await?,
                    None => {}
                }
            } else if dir == "." {
                let cur_idx = ctx.cursor as usize - 1;
                let cur = match self.file_items.get(cur_idx) {
                    Some(i) => i,
                    None => {
                        Err(ArgError::new("invalid cursor pos"))?;
                        return Ok(());
                    }
                };
                let cur_path_str = cur.path.to_str().unwrap();
                let cmd = if self.is_item_opened(cur_path_str) {
                    format!("cd {}", cur_path_str)
                } else {
                    format!("cd {}", dir)
                };
                nvim.command(&cmd).await?
            } else {
                self.change_root(dir, nvim).await?;
            }
        }
        Ok(())
    }
    /// Open like :drop
    pub async fn action_update_git_map<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _args: Value,
        _ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.columns.contains(&ColumnType::GIT) {
            self.update_git_map();
            self.redraw_subtree(nvim, 0, false).await?;
        }
        Ok(())
    }

    /// Open like :drop
    pub async fn action_drop<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        args: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let info: String;
        let should_change_root;
        if let Some(cur) = self.file_items.get(ctx.cursor as usize - 1) {
            info = cur.path.to_str().unwrap().to_owned();
            if cur.metadata.is_dir() {
                should_change_root = true;
            } else {
                should_change_root = false;
            }
        } else {
            return Err(Box::new(ArgError::new("drop: invalid cursor position")));
        }
        if should_change_root {
            self.change_root(&info, nvim).await?;
        } else {
            nvim.execute_lua("tree.drop(...)", vec![args, Value::from(info)])
                .await?;
        }
        Ok(())
    }

    pub async fn close_tree<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        idx: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // skip the root
        if idx == 0 {
            return Ok(());
        }

        // get the current
        let target = match self.file_items.get(idx) {
            Some(fi) => fi,
            None => {
                return Err(Box::new(ArgError::from_string(format!(
                    "Index out of bound: {}",
                    idx
                ))));
            }
        }
        .clone();
        let path_str = match target.path.to_str() {
            Some(path) => path,
            None => {
                return Err(Box::new(ArgError::new("filename error")));
            }
        };
        let is_opened = match self.expand_store.get(path_str) {
            Some(v) => *v,
            None => false,
        };
        if target.metadata.is_dir() && is_opened {
            self.expand_store.remove(path_str);
            let start = idx + 1;
            let base_level = target.level;
            let mut end = start;
            for fi in &self.file_items[start..] {
                if fi.level <= base_level {
                    break;
                }
                end += 1;
            }
            self.remove_items_and_cells(start, end)?;
            self.update_cells(idx, idx + 1);
            let ret = vec![self.makeline(idx)];
            self.buf_set_lines(nvim, idx as i64, end as i64, true, ret)
                .await?;
            self.hl_lines(&nvim, idx, idx + 1).await?;
        }

        Ok(())
    }

    pub async fn open_tree<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        idx: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // don't open root item
        if idx == 0 {
            return Ok(());
        }
        let cur = match self.file_items.get(idx) {
            Some(fi) => fi,
            None => {
                return Err(Box::new(ArgError::from_string(format!(
                    "Index out of bound: {}",
                    idx
                ))));
            }
        }
        .clone();
        let path_str = match cur.path.to_str() {
            Some(path) => path,
            None => {
                return Err(Box::new(ArgError::new("filename error")));
            }
        };
        let is_opened = match self.expand_store.get(path_str) {
            Some(v) => *v,
            None => false,
        };

        if cur.metadata.is_dir() && !is_opened {
            let mut child_fileitem = Vec::new();
            self.entry_info_recursively_sync(cur.clone(), &mut child_fileitem, idx + 1)?;
            self.expand_store.insert(path_str.to_owned(), true);
            // icon should be open
            self.update_cells(idx, idx + 1);
            let child_item_size = child_fileitem.len();
            self.insert_items_and_cells(idx + 1, child_fileitem)?;
            // update lines
            let end = idx + child_item_size + 1;
            let ret = (idx..end).map(|i| self.makeline(i)).collect();
            self.buf_set_lines(nvim, idx as i64, (idx + 1) as i64, true, ret)
                .await?;
            self.hl_lines(&nvim, idx, idx + 1 + child_item_size).await?;
        }
        Ok(())
    }
    pub async fn action_close_tree<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _args: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let idx = ctx.cursor as usize - 1;
        let target = match self.file_items.get(idx) {
            Some(fi) => fi,
            None => {
                return Err(Box::new(ArgError::from_string(format!(
                    "item not found: {}",
                    idx
                ))));
            }
        };
        if target.metadata.is_dir() && self.is_item_opened(target.path.to_str().unwrap()) {
            self.close_tree(nvim, idx).await
        } else if let Some(p) = target.parent.clone() {
            self.close_tree(nvim, p.id).await?;
            match nvim
                .call("cursor", vec![Value::from(p.id + 1), Value::from(1)])
                .await?
            {
                Err(e) => error!("{:?}", e),
                _ => {}
            };
            Ok(())
        } else {
            Ok(())
        }
    }

    pub async fn action_open_or_close_tree<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _args: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let idx = ctx.cursor as usize - 1;
        let target = match self.file_items.get(idx) {
            Some(fi) => fi,
            None => {
                return Err(Box::new(ArgError::from_string(format!(
                    "item not found: {}",
                    idx
                ))));
            }
        };

        if target.metadata.is_dir() && self.is_item_opened(target.path.to_str().unwrap()) {
            self.close_tree(nvim, idx).await?;
        } else {
            self.open_tree(nvim, idx).await?;
        }
        Ok(())
    }

    pub async fn action_open_directory<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _args: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let idx = ctx.cursor as usize - 1;
        let target = match self.file_items.get(idx) {
            Some(fi) => fi,
            None => {
                return Err(Box::new(ArgError::from_string(format!(
                    "item not found: {}",
                    idx
                ))));
            }
        };
        if target.metadata.is_dir() && idx != 0 {
            let target_path = target.path.to_str().unwrap().to_owned();
            self.change_root(&target_path, nvim).await?;
        }
        Ok(())
    }

    pub async fn action_open_tree<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _args: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let idx = ctx.cursor as usize - 1;
        self.open_tree(nvim, idx).await
    }

    pub fn update_cells(&mut self, sl: usize, el: usize) {
        // self.update_git_map();
        let cells = self.make_cells(&self.file_items[sl..el], sl == 0);
        for (col, cells) in cells {
            if !self.col_map.contains_key(&col) {
                self.col_map.insert(col.clone(), Vec::new());
            }
            self.col_map.get_mut(&col).unwrap().splice(sl..el, cells);
        }
    }

    pub fn get_context_value(&self, cursor: usize) -> Value {
        let idx = cursor - 1;
        let ft = self.file_items.get(idx).unwrap();
        info!("get context of: {:?}", ft.path);
        Value::Map(vec![
            (
                Value::from("is_directory"),
                Value::from(ft.metadata.is_dir()),
            ),
            (
                Value::from("is_opened_tree"),
                Value::from(self.is_item_opened(ft.path.to_str().unwrap())),
            ),
            (Value::from("level"), Value::from(ft.level)),
        ])
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
        let root_path = absolute_path(path)?;
        // if we have loaded git repo previously, we need to update
        // it. Otherwise we won't do a hard reload in the future
        if self.git_repo.is_some() {
            self.init_git_repo(&root_path);
            // self.update_git_map();
        }
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

        self.targets.clear();
        self.col_map.clear();
        self.file_items.clear();

        let filemeta = std::fs::metadata(root_path_str)?;
        let mut fileitems = vec![Arc::new(FileItem::new(root_path, filemeta, 0))];

        // recursively what the directory and build up the tree
        self.entry_info_recursively_sync(fileitems[0].clone(), &mut fileitems, 1)?;

        self.insert_items_and_cells(0, fileitems)?;

        let ret = (0..self.file_items.len())
            .map(|i| self.makeline(i))
            .collect();
        self.buf_set_lines(nvim, 0, -1, true, ret).await?;
        self.hl_lines(&nvim, 0, self.file_items.len()).await?;
        if let Some(v) = last_cursor {
            let win = Window::new(Value::from(0), nvim.clone());
            let cursor_pos = if v as usize >= self.file_items.len() {
                0_i64
            } else {
                v as i64
            };
            match win.set_cursor((cursor_pos, 0)).await {
                Ok(_) => {}
                Err(e) => warn!("Fail to set cursor position {}: {:?}", cursor_pos, e),
            };
        }
        Ok(())
    }

    fn make_cells(
        &self,
        items: &[FileItemPtr],
        first_item_is_root: bool,
    ) -> Vec<(ColumnType, Vec<ColumnCell>)> {
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
                let mut cell = ColumnCell::new(self, fileitem, col.clone(), is_root);
                cell.byte_start = byte_start;
                cell.byte_end = byte_start + cell.text.len();
                cell.col_start = start;
                cell.col_end = start + UnicodeWidthStr::width(cell.text.as_str());
                // NOTE: alignment
                if *col == ColumnType::FILENAME {
                    let stop = KSTOP as i64 - cell.col_end as i64;
                    if stop > 0 {
                        cell.col_end += stop as usize;
                        cell.byte_end += stop as usize;
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

    /*
    fn remove_cells(&mut self, start: usize, end: usize) {
        for (_, val) in self.col_map.iter_mut() {
            val.splice(start..end, vec![]);
        }
    }
    */

    fn remove_items_and_cells(&mut self, start: usize, end: usize) -> Result<(), ArgError> {
        // remove the items in between
        for (_, val) in self.col_map.iter_mut() {
            val.splice(start..end, vec![]);
        }
        self.file_items.splice(start..end, vec![]);
        for i in start..end {
            self.selected_items.remove(&i);
        }

        // items after the deleted
        if start < self.file_items.len() {
            for i in start..self.file_items.len() {
                let fi = self.file_items[i].as_ref();
                // replace the old id with the new id for selected items
                if self.selected_items.remove(&fi.id) {
                    self.selected_items.insert(i);
                }
                // This should be safe here, &mut self (guaranteed by rust) is the only mutable borrowing
                // and fileitem won't live outside self (it's private, and the only place referring
                // fileitem other than the fileitem list is the "parent" field of other fileitem
                unsafe {
                    (&mut *(fi as *const FileItem as *mut FileItem)).id = i;
                }
            }
        }

        Ok(())
    }

    // insert at the pos
    fn insert_items_and_cells(
        &mut self,
        pos: usize,
        items: Vec<FileItemPtr>,
    ) -> Result<(), ArgError> {
        if pos > self.file_items.len() {
            return Err(ArgError::new("pos larger than the fileitem size"));
        }
        let is_first_item_root = pos == 0;
        // insert items
        let size_to_insert = items.len();
        self.file_items.splice(pos..pos, items.iter().cloned());
        // update the indices
        if pos + size_to_insert < self.file_items.len() {
            for i in pos + size_to_insert..self.file_items.len() {
                let fi = self.file_items[i].as_ref();
                if self.selected_items.remove(&fi.id) {
                    self.selected_items.insert(i);
                }
                // This should be safe here, &mut self is the only mutable borrowing
                // and fileitem won't live outside self
                unsafe {
                    (&mut *(fi as *const FileItem as *mut FileItem)).id = i;
                }
            }
        }

        // make cells
        let cells = self.make_cells(&items, is_first_item_root);
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
        let buf = Buffer::new(self.bufnr.clone(), nvim.clone());
        buf.set_option("modifiable", Value::from(true)).await?;
        buf.set_lines(start, end, strict, replacement).await?;
        buf.set_option("modifiable", Value::from(false)).await?;
        Ok(())
    }

    // NOTE: tests show that the sync version is much faster than the async version
    // using tokio::fs
    fn entry_info_recursively_sync<'a>(
        &'a self,
        item: Arc<FileItem>,
        fileitem_lst: &'a mut Vec<FileItemPtr>,
        mut start_id: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut entries: Vec<_> = std::fs::read_dir(&item.path)?
            .map(|x| x.unwrap())
            .filter(|x| {
                self.config.show_ignored_files
                    || !(x.file_name().to_str().unwrap().starts_with('.'))
            })
            .map(|x| {
                let meta = x.metadata().unwrap();
                (x, meta)
            })
            .collect();
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
            let mut fileitem = FileItem::new(absolute_path(entry.0.path())?, entry.1, start_id);
            start_id += 1;
            fileitem.level = level;
            fileitem.parent = Some(item.clone());
            if i == count - 1 {
                fileitem.last = true;
            }
            i += 1;
            if let Some(expand) = self.expand_store.get(fileitem.path.to_str().unwrap()) {
                if *expand {
                    let ft_ptr = Arc::new(fileitem);
                    fileitem_lst.push(ft_ptr.clone());
                    start_id =
                        self.entry_info_recursively_sync(ft_ptr.clone(), fileitem_lst, start_id)?
                } else {
                    fileitem_lst.push(Arc::new(fileitem));
                }
            } else {
                fileitem_lst.push(Arc::new(fileitem));
            }
        }
        Ok(start_id)
    }

    /*
    fn entry_info_recursively<'a>(
        &'a self,
        item: Arc<FileItem>,
        fileitem_lst: &'a mut Vec<FileItemPtr>,
        mut start_id: usize,
    ) -> Pin<Box<dyn Future<Output = Result<usize, Box<dyn std::error::Error>>> + 'a + Send>> {
        Box::pin(async move {
            let mut read_dir = tokio::fs::read_dir(&item.path).await?;
            let mut dir_entries = Vec::new();
            // filter: dirs, files, no dot and dot dot
            while let Some(entry) = read_dir.next_entry().await? {
                // skip hidden file or dot or dot dot
                if fs_utils::is_dot_or_dotdot(&entry)
                    || (!self.config.show_ignored_files && fs_utils::is_hidden(&entry))
                {
                    continue;
                }
                dir_entries.push(entry);
                // entries.push((entry, metadata));
            }
            if dir_entries.len() <= 0 {
                return Ok(start_id);
            }
            let metadata = futures::future::join_all(dir_entries.iter().map(|x| x.metadata()))
                .await
                .into_iter()
                .map(|x| x.unwrap());
            let mut entries: Vec<_> = dir_entries.into_iter().zip(metadata.into_iter()).collect();
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
                let mut fileitem = FileItem::new(
                    tokio::fs::canonicalize(entry.0.path()).await?,
                    entry.1,
                    start_id,
                );
                start_id += 1;
                fileitem.level = level;
                fileitem.parent = Some(item.clone());
                if i == count - 1 {
                    fileitem.last = true;
                }
                i += 1;
                if let Some(expand) = self.expand_store.get(fileitem.path.to_str().unwrap()) {
                    if *expand {
                        let ft_ptr = Arc::new(fileitem);
                        fileitem_lst.push(ft_ptr.clone());
                        start_id = self.entry_info_recursively_sync(
                            ft_ptr.clone(),
                            fileitem_lst,
                            start_id,
                        )?;
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
    */

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
        let mut hl_args = Vec::<Value>::new();
        let icon_ns_id = self.icon_ns_id;
        for i in sl..el {
            for col in &self.config.columns {
                let cell = &self.col_map.get(col).unwrap()[i];
                if let Some(hl_group) = cell.hl_group.clone() {
                    // let buf = Buffer::new(self.bufnr.clone(), nvim.clone());
                    let start = cell.byte_start as i64;
                    let end = (cell.byte_start + cell.text.len()) as i64;
                    hl_args.push(Value::from(hl_group));
                    hl_args.push(Value::from(start));
                    hl_args.push(Value::from(end));
                    hl_args.push(Value::from(i));
                    // async_std::task::spawn(async move {
                    //     let hl_group = hl_group;
                    //     buf.add_highlight(icon_ns_id, &hl_group, i as i64, start, end)
                    //         .await
                    //         .unwrap();
                    // });
                }
            }
        }
        let args = vec![
            self.bufnr.clone(),
            Value::from(icon_ns_id),
            Value::from(hl_args),
        ];
        let nvim_c = nvim.clone();
        async_std::task::spawn(async move {
            nvim_c
                .execute_lua("tree.hl_lines(...)", args)
                .await
                .unwrap();
        });
        Ok(())
    }

    pub async fn action_copy<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            (*CLIPBOARD_MODE.write().await) = ClipboardMode::COPY;
        }
        self.copy_or_move(ctx).await?;
        nvim.execute_lua(
            "tree.print_message(...)",
            vec![Value::from("Copy to clipboard")],
        )
        .await?;
        Ok(())
    }

    pub async fn action_move<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            (*CLIPBOARD_MODE.write().await) = ClipboardMode::MOVE;
        }
        self.copy_or_move(ctx).await?;
        nvim.execute_lua(
            "tree.print_message(...)",
            vec![Value::from("Move to clipboard")],
        )
        .await?;
        Ok(())
    }

    pub async fn copy_or_move(&self, ctx: Context) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = CLIPBOARD.write().await;
        clipboard.clear();
        if self.selected_items.is_empty() {
            clipboard.push(self.file_items[ctx.cursor as usize - 1].path.clone());
        } else {
            clipboard.extend(
                self.selected_items
                    .iter()
                    .map(|x| self.file_items[*x].path.clone()),
            )
        }

        Ok(())
    }
    pub async fn action_paste<W: AsyncWrite + Sync + Send + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        _arg: Value,
        ctx: Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let clipboard_empty = { CLIPBOARD.read().await.is_empty() };
        if clipboard_empty {
            nvim.execute_lua(
                "tree.print_message(...)",
                vec![Value::from("Nothing in clipboard")],
            )
            .await?;
            return Ok(());
        }
        let items: Vec<_> = { CLIPBOARD.read().await.iter().map(|x| x.clone()).collect() };
        for item in items {
            if !item.exists() {
                continue;
            }
            let cur = self.file_items[ctx.cursor as usize - 1].as_ref();
            let dest_fname = item.file_name().unwrap().to_str().unwrap().to_owned();
            let cur_dir = cur.path.parent().unwrap().to_path_buf();
            let mut dest_file = cur_dir.clone();
            dest_file.push(PathBuf::from(dest_fname).as_path());
            info!("dest_file: {:?}", dest_file);
            if dest_file.exists() {
                let dest_meta = std::fs::metadata(&dest_file)?;
                let src_meta = std::fs::metadata(&item)?;
                let dest = Value::from(vec![
                    (
                        Value::from("mtime"),
                        Value::from(
                            dest_meta
                                .modified()?
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)?
                                .as_secs(),
                        ),
                    ),
                    (
                        Value::from("path"),
                        Value::from(dest_file.as_os_str().to_str().unwrap()),
                    ),
                    (Value::from("size"), Value::from(dest_meta.len())),
                ]);
                let src = Value::from(vec![
                    (
                        Value::from("mtime"),
                        Value::from(
                            src_meta
                                .modified()?
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)?
                                .as_secs(),
                        ),
                    ),
                    (
                        Value::from("path"),
                        Value::from(item.as_os_str().to_str().unwrap()),
                    ),
                    (Value::from("size"), Value::from(src_meta.len())),
                ]);
                nvim.execute_lua(
                    "tree.pre_paste(...)",
                    vec![
                        Value::from(vec![self.bufnr.clone(), Value::from(ctx.cursor as u64 - 1)]),
                        src,
                        dest,
                    ],
                )
                .await?;
            } else {
                self.func_paste(
                    nvim,
                    ctx.cursor - 1,
                    item.as_os_str().to_str().unwrap(),
                    dest_file.as_os_str().to_str().unwrap(),
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn func_paste<W: AsyncWrite + Send + Sync + Unpin + 'static>(
        &mut self,
        nvim: &Neovim<W>,
        idx: u64,
        src: &str,
        dest: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mode;
        {
            let clipboard_mode = CLIPBOARD_MODE.read().await;
            match *clipboard_mode {
                ClipboardMode::COPY => {
                    mode = ClipboardMode::COPY;
                }
                ClipboardMode::MOVE => {
                    mode = ClipboardMode::MOVE;
                }
            }
        }
        let from_path = Path::new(src);
        let to_path = Path::new(dest);
        let is_dir = std::fs::metadata(from_path).unwrap().is_dir();
        match mode {
            ClipboardMode::COPY => {
                if is_dir {
                    fs_extra::dir::copy(&from_path, &to_path, &fs_extra::dir::CopyOptions::new())?;
                } else {
                    std::fs::copy(from_path, to_path)?;
                }
                let idx_to_redraw =
                    if let Some(parent) = self.file_items[idx as usize].parent.as_ref() {
                        parent.id
                    } else {
                        0
                    };
                self.redraw_subtree(nvim, idx_to_redraw, true).await?;
            }
            ClipboardMode::MOVE => {
                std::fs::rename(from_path, to_path)?;
                self.redraw_subtree(nvim, 0, true).await?;
            }
        }
        Ok(())
    }
}
