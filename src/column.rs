use crate::tree::Tree;
use std::convert::From;
use std::ffi::OsStr;
use std::fs::Metadata;
#[derive(Eq, PartialEq, Clone)]
enum Icon {
    FolderClosed,
    FolderOpened,
    FolderSymlink,
    File,
    FileSymlink,
    FileHidden,
    Excel,
    Word,
    Ppt,
    Stylus,
    Sass,
    Html,
    Xml,
    Ejs,
    Css,
    Webpack,
    Markdown,
    Json,
    Javascript,
    Javascriptreact,
    Ruby,
    Php,
    Python,
    Coffee,
    Mustache,
    Conf,
    Image,
    Ico,
    Twig,
    C,
    H,
    Haskell,
    Lua,
    Java,
    Terminal,
    Ml,
    Diff,
    Sql,
    Clojure,
    Edn,
    Scala,
    Go,
    Dart,
    Firefox,
    Vs,
    Perl,
    Rss,
    Fsharp,
    Rust,
    Dlang,
    Erlang,
    Elixir,
    Mix,
    Vim,
    Ai,
    Psd,
    Psb,
    Typescript,
    Typescriptreact,
    Julia,
    Puppet,
    Vue,
    Swift,
    Gitconfig,
    Bashrc,
    Favicon,
    Docker,
    Gruntfile,
    Gulpfile,
    Dropbox,
    License,
    Procfile,
    Jquery,
    Angular,
    Backbone,
    Requirejs,
    Materialize,
    Mootools,
    Vagrant,
    Svg,
    Font,
    Text,
    Archive,
    Unknonwn,
}

impl Into<usize> for Icon {
    fn into(self) -> usize {
        self as usize
    }
}

impl From<&str> for Icon {
    fn from(s: &str) -> Icon {
        match s {
            "styl" => Icon::Stylus,
            "sass" => Icon::Sass,
            "scss" => Icon::Sass,
            "htm" => Icon::Html,
            "html" => Icon::Html,
            "slim" => Icon::Html,
            "xml" => Icon::Xml,
            "xaml" => Icon::Xml,
            "ejs" => Icon::Ejs,
            "css" => Icon::Css,
            "less" => Icon::Css,
            "md" => Icon::Markdown,
            "mdx" => Icon::Markdown,
            "markdown" => Icon::Markdown,
            "rmd" => Icon::Markdown,
            "json" => Icon::Json,
            "js" => Icon::Javascript,
            "es6" => Icon::Javascript,
            "jsx" => Icon::Javascriptreact,
            "rb" => Icon::Ruby,
            "ru" => Icon::Ruby,
            "php" => Icon::Php,
            "py" => Icon::Python,
            "pyc" => Icon::Python,
            "pyo" => Icon::Python,
            "pyd" => Icon::Python,
            "coffee" => Icon::Coffee,
            "mustache" => Icon::Mustache,
            "hbs" => Icon::Mustache,
            "config" => Icon::Conf,
            "conf" => Icon::Conf,
            "ini" => Icon::Conf,
            "yml" => Icon::Conf,
            "yaml" => Icon::Conf,
            "toml" => Icon::Conf,
            "jpg" => Icon::Image,
            "jpeg" => Icon::Image,
            "bmp" => Icon::Image,
            "png" => Icon::Image,
            "gif" => Icon::Image,
            "ico" => Icon::Ico,
            "twig" => Icon::Twig,
            "cpp" => Icon::C,
            "c++" => Icon::C,
            "cxx" => Icon::C,
            "cc" => Icon::C,
            "cp" => Icon::C,
            "c" => Icon::C,
            "h" => Icon::H,
            "hpp" => Icon::H,
            "hxx" => Icon::H,
            "hs" => Icon::Haskell,
            "lhs" => Icon::Haskell,
            "lua" => Icon::Lua,
            "java" => Icon::Java,
            "jar" => Icon::Java,
            "sh" => Icon::Terminal,
            "fish" => Icon::Terminal,
            "bash" => Icon::Terminal,
            "zsh" => Icon::Terminal,
            "ksh" => Icon::Terminal,
            "csh" => Icon::Terminal,
            "awk" => Icon::Terminal,
            "ps1" => Icon::Terminal,
            "bat" => Icon::Terminal,
            "cmd" => Icon::Terminal,
            "ml" => Icon::Ml,
            "mli" => Icon::Ml,
            "diff" => Icon::Diff,
            "db" => Icon::Sql,
            "sql" => Icon::Sql,
            "dump" => Icon::Sql,
            "accdb" => Icon::Sql,
            "clj" => Icon::Clojure,
            "cljc" => Icon::Clojure,
            "cljs" => Icon::Clojure,
            "edn" => Icon::Edn,
            "scala" => Icon::Scala,
            "go" => Icon::Go,
            "dart" => Icon::Dart,
            "xul" => Icon::Firefox,
            "sln" => Icon::Vs,
            "suo" => Icon::Vs,
            "pl" => Icon::Perl,
            "pm" => Icon::Perl,
            "t" => Icon::Perl,
            "rss" => Icon::Rss,
            "f#" => Icon::Fsharp,
            "fsscript" => Icon::Fsharp,
            "fsx" => Icon::Fsharp,
            "fs" => Icon::Fsharp,
            "fsi" => Icon::Fsharp,
            "rs" => Icon::Rust,
            "rlib" => Icon::Rust,
            "d" => Icon::Dlang,
            "erl" => Icon::Erlang,
            "hrl" => Icon::Erlang,
            "ex" => Icon::Elixir,
            "exs" => Icon::Elixir,
            "exx" => Icon::Elixir,
            "leex" => Icon::Elixir,
            "vim" => Icon::Vim,
            "ai" => Icon::Ai,
            "psd" => Icon::Psd,
            "psb" => Icon::Psd,
            "ts" => Icon::Typescript,
            "tsx" => Icon::Javascriptreact,
            "jl" => Icon::Julia,
            "pp" => Icon::Puppet,
            "vue" => Icon::Vue,
            "swift" => Icon::Swift,
            "xcplayground" => Icon::Swift,
            "svg" => Icon::Svg,
            "otf" => Icon::Font,
            "ttf" => Icon::Font,
            "fnt" => Icon::Font,
            "txt" => Icon::Text,
            "text" => Icon::Text,
            "zip" => Icon::Archive,
            "tar" => Icon::Archive,
            "gz" => Icon::Archive,
            "gzip" => Icon::Archive,
            "rar" => Icon::Archive,
            "7z" => Icon::Archive,
            "iso" => Icon::Archive,
            "doc" => Icon::Word,
            "docx" => Icon::Word,
            "docm" => Icon::Word,
            "csv" => Icon::Excel,
            "xls" => Icon::Excel,
            "xlsx" => Icon::Excel,
            "xlsm" => Icon::Excel,
            "ppt" => Icon::Ppt,
            "pptx" => Icon::Ppt,
            "pptm" => Icon::Ppt,
            _ => Icon::Unknonwn,
        }
    }
}

pub static GUI_COLORS: &[&'static str] = &[
    "#905532", "#3AFFDB", "#689FB6", "#44788E", "#834F79", "#834F79", "#AE403F", "#F5C06F",
    "#F09F17", "#D4843E", "#F16529", "#CB6F6F", "#EE6E73", "#8FAA54", "#31B53E", "#FFFFFF",
];

pub static ICONS: &[&[&'static str]] = &[
    &["", "#00afaf"],
    &["", "#00afaf"],
    &["", "#00afaf"],
    &["", "#999999"],
    &["", "#999999"],
    &["﬒", "#999999"],
    &["", "#207245"],
    &["", "#185abd"],
    &["", "#cb4a32"],
    &["", "#8dc149"],
    &["", "#f55385"],
    &["", "#e37933"],
    &["", "#cbcb41"],
    &["", "#519aba"],
    &["ﰩ", "#519aba"],
    &["", "#519aba"],
    &["", "#cbcb41"],
    &["", "#cbcb41"],
    &["", "#519aba"],
    &["", "#cc3e44"],
    &["", "#a074c4"],
    &["", "#519aba"],
    &["", "#cbcb41"],
    &["", "#e37933"],
    &["", "#6d8086"],
    &["", "#a074c4"],
    &["", "#cbcb41"],
    &["", "#8dc149"],
    &["", "#519aba"],
    &["", "#a074c4"],
    &["", "#a074c4"],
    &["", "#519aba"],
    &["", "#cc3e44"],
    &["", "#4d5a5e"],
    &["λ", "#e37933"],
    &["", "#41535b"],
    &["", "#f55385"],
    &["", "#8dc149"],
    &["", "#519aba"],
    &["", "#cc3e44"],
    &["", "#519aba"],
    &["", "#03589C"],
    &["", "#e37933"],
    &["", "#854CC7"],
    &["", "#519aba"],
    &["", "FB9D3B"],
    &["", "#519aba"],
    &["", "#519aba"],
    &["", "#cc3e44"],
    &["", "#A90533"],
    &["", "#a074c4"],
    &["", "#cc3e44"],
    &["", "#019833"],
    &["", "#cbcb41"],
    &["", "#519aba"],
    &["", "#519aba"],
    &["", "#519aba"],
    &["", "#519aba"],
    &["", "#a074c4"],
    &["", "#cbcb41"],
    &["﵂", "#8dc149"],
    &["", "#e37933"],
    &["", "#41535b"],
    &["", "#4d5a5e"],
    &["", "#cbcb41"],
    &["", "#519aba"],
    &["", "#e37933"],
    &["", "#cc3e44"],
    &["", "#0061FE"],
    &["", "#cbcb41"],
    &["", "#a074c4"],
    &["", "#1B75BB"],
    &["", "#E23237"],
    &["", "#0071B5"],
    &["", "#F44A41"],
    &["", "#EE6E73"],
    &["", "#ECECEC"],
    &["", "#1563FF"],
    &["ﰟ", "#FFB13B"],
    &["", "#999999"],
    &["", "#999999"],
    &["", "#cc3e44"],
];

pub static GIT_INDICATORS: &[&[&'static str]] = &[
    &["✭", "#FFFFFF"], // Untracked
    &["✹", "#fabd2f"], // Modified
    &["✚", "#b8bb26"], // Staged
    &["➜", "#fabd2f"], // Renamed
    &["☒", "#FFFFFF"], // Ignored
    &["═", "#fb4934"], // Unmerged
    &["✖", "#fb4934"], // Deleted
    &["?", "#FFFFFF"],   // Unknown
];

static READ_ONLY_ICON: &'static str = "✗";
static SELECTED_ICON: &'static str = "✓";

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum ColumnType {
    MARK,
    INDENT,
    GIT,
    ICON,
    FILENAME,
    SIZE,
    TIME,
}

impl Into<u8> for ColumnType {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<&str> for ColumnType {
    fn from(s: &str) -> Self {
        match s {
            "mark" => ColumnType::MARK,
            "ident" => ColumnType::INDENT,
            "git" => ColumnType::GIT,
            "icon" => ColumnType::ICON,
            "filename" => ColumnType::FILENAME,
            "size" => ColumnType::SIZE,
            "time" => ColumnType::TIME,
            _ => panic!("Error! unknown column type"),
        }
    }
}

pub enum GuiColor {
    BROWN,
    AQUA,
    BLUE,
    DARKBLUE,
    PURPLE,
    LIGHTPURPLE,
    RED,
    BEIGE,
    YELLOW,
    ORANGE,
    DARKORANGE,
    PINK,
    SALMON,
    GREEN,
    LIGHTGREEN,
    WHITE,
}

impl Into<usize> for GuiColor {
    fn into(self) -> usize {
        self as usize
    }
}

pub enum GitStatus {
    Untracked,
    Modified,
    Staged,
    Renamed,
    Ignored,
    Unmerged,
    Deleted,
    Unknown,
}

pub struct FileItem {
    pub path: std::path::PathBuf,
    pub metadata: Metadata,
    pub level: usize,
    pub opened_tree: bool,
    pub selected: bool,
    pub parent: Option<usize>, // the index of the parent in the tree list
    pub last: bool,
    // pub git_map: HashMap<String, GitStatus>,
}

impl FileItem {
    pub fn new(path: std::path::PathBuf, metadata: Metadata) -> Self {
        Self {
            path,
            metadata,
            level: 0,
            opened_tree: false,
            selected: false,
            parent: None,
            last: false,
        }
    }

    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(OsStr::to_str)
    }
}

pub struct Cell {
    pub col_start: usize,
    pub col_end: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub text: String,
    pub color: usize,
}

impl Cell {
    pub fn new(tree: &Tree, fileitem: &FileItem, ty: ColumnType) -> Self {
        let mut text = String::new();
        let mut color: usize = Icon::Unknonwn.into();
        match ty {
            ColumnType::MARK => {
                if fileitem.metadata.permissions().readonly() {
                    text = String::from(READ_ONLY_ICON);
                } else {
                    text = String::from(" ");
                }
            }
            ColumnType::INDENT => {
                let mut icon_idx: i32 = -1;
                let mut indent_idx: i32 = -1;
                for (i, col) in tree.config.columns.iter().enumerate() {
                    if *col == ColumnType::ICON {
                        icon_idx = i as i32;
                    }
                    if *col == ColumnType::INDENT {
                        indent_idx = i as i32;
                    }
                }
                let margin = icon_idx - indent_idx - 1;
                let margin_val = if margin >= 0 { margin as usize } else { 0usize };
                let prefix = unsafe { String::from_utf8_unchecked(vec![b' '; margin_val * 2]) };
                let mut inversed_elements: Vec<&str> = Vec::new();
                if fileitem.level > 0 {
                    if fileitem.last {
                        inversed_elements.push("└ ");
                    } else {
                        inversed_elements.push("│ ");
                    }
                    inversed_elements.push(prefix.as_str());
                    let max_level = fileitem.level - 1;
                    let mut i = 0;
                    let mut pf_idx = fileitem.parent;
                    while let Some(pf_idx_v) = pf_idx {
                        if i >= max_level {
                            break;
                        }
                        let pf = &tree.get_fileitem(pf_idx_v);
                        if pf.last {
                            inversed_elements.push("  ");
                        } else {
                            inversed_elements.push("│ ");
                        }

                        pf_idx = pf.parent;
                        i = i + 1;
                    }
                }
                text = String::new();
                while let Some(top) = inversed_elements.pop() {
                    text.push_str(top);
                }
            }
            ColumnType::GIT => {
                // unimplemented
            }
            ColumnType::ICON => {
                if fileitem.metadata.is_dir() {
                    text = String::from(" ");
                    if fileitem.opened_tree {
                        color = Icon::FolderOpened.into();
                    } else if fileitem.metadata.file_type().is_symlink() {
                        color = Icon::FolderSymlink.into();
                    } else {
                        color = Icon::FolderClosed.into();
                    }
                } else {
                    let extension_icon = match fileitem.extension() {
                        Some(extension) => Icon::from(extension),
                        None => Icon::Unknonwn,
                    };
                    if extension_icon != Icon::Unknonwn {
                        let icon_idx: usize = extension_icon.into();
                        text = String::from(ICONS[icon_idx][0]);
                        color = icon_idx;
                    } else {
                        text = String::from(" ");
                        color = Icon::File.into();
                    }
                }
            }
            ColumnType::FILENAME => {
                color = GuiColor::YELLOW.into();
                text = String::from(fileitem.path.file_name().and_then(OsStr::to_str).unwrap());
                if fileitem.metadata.is_dir() {
                    text.push('/');
                    color = GuiColor::BLUE.into();
                }
            }
            ColumnType::SIZE => {}
            ColumnType::TIME => {}
        };
        Self {
            col_start: 0,
            col_end: 0,
            byte_start: 0,
            byte_end: 0,
            text,
            color,
        }
    }
}
