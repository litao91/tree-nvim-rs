use crate::column::ColumnType;

pub enum SplitType {
    Vertical,
    Horizontal,
    No,
    Tab,
    Floating,
}

// State parameters for Tree
pub struct Config {
    pub auto_cd: bool,
    pub auto_recursive_level: u16,
    pub columns: Vec<ColumnType>,
    pub ignore_files: String,
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
            ignore_files: String::new(),
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
