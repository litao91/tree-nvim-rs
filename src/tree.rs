use crate::column::ColumnType;
use log::*;
use nvim_rs::Value;
use std::collections::HashMap;
use std::convert::From;

pub enum SplitType {
    Vertical,
    Horizontal,
    No,
    Tab,
    Floating,
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
    pub fn update(&mut self, cfg: HashMap<String, Value>) {
        for (k, v) in cfg {
            match k.as_str() {
                "auto_recursive_level" => self.auto_recursive_level = v.as_u64().unwrap() as u16,
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

                    }
                }
                _ => error!("Unsupported member: {}", k),
            };
        }
    }
}

pub struct Tree {
    bufnr: (i8, Vec<u8>), // use bufnr to avoid tedious generic code
    icon_ns_id: i64,
    config: Config,
}
impl Tree {
    pub fn new(bufnr: (i8, Vec<u8>), icon_ns_id: i64) -> Self {
        Self {
            bufnr,
            icon_ns_id,
            config: Default::default(),
        }
    }
}
