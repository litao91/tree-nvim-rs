use crate::tree::Tree;
use std::convert::From;
use std::ffi::OsStr;
use std::fs::Metadata;

#[derive(Eq, PartialEq, Clone)]
pub enum Icon {
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
    Unknown,
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
            _ => Icon::Unknown,
        }
    }
}

impl Icon {
    pub fn hl_group_name(&self) -> &str {
        match *self {
            Icon::FolderClosed => "tree_icon_FolderClosed",
            Icon::FolderOpened => "tree_icon_FolderOpened",
            Icon::FolderSymlink => "tree_icon_FolderSymlink",
            Icon::File => "tree_icon_File",
            Icon::FileSymlink => "tree_icon_FileSymlink",
            Icon::FileHidden => "tree_icon_FileHidden",
            Icon::Excel => "tree_icon_Excel",
            Icon::Word => "tree_icon_Word",
            Icon::Ppt => "tree_icon_Ppt",
            Icon::Stylus => "tree_icon_Stylus",
            Icon::Sass => "tree_icon_Sass",
            Icon::Html => "tree_icon_Html",
            Icon::Xml => "tree_icon_Xml",
            Icon::Ejs => "tree_icon_Ejs",
            Icon::Css => "tree_icon_Css",
            Icon::Webpack => "tree_icon_Webpack",
            Icon::Markdown => "tree_icon_Markdown",
            Icon::Json => "tree_icon_Json",
            Icon::Javascript => "tree_icon_Javascript",
            Icon::Javascriptreact => "tree_icon_Javascriptreact",
            Icon::Ruby => "tree_icon_Ruby",
            Icon::Php => "tree_icon_Php",
            Icon::Python => "tree_icon_Python",
            Icon::Coffee => "tree_icon_Coffee",
            Icon::Mustache => "tree_icon_Mustache",
            Icon::Conf => "tree_icon_Conf",
            Icon::Image => "tree_icon_Image",
            Icon::Ico => "tree_icon_Ico",
            Icon::Twig => "tree_icon_Twig",
            Icon::C => "tree_icon_C",
            Icon::H => "tree_icon_H",
            Icon::Haskell => "tree_icon_Haskell",
            Icon::Lua => "tree_icon_Lua",
            Icon::Java => "tree_icon_Java",
            Icon::Terminal => "tree_icon_Terminal",
            Icon::Ml => "tree_icon_Ml",
            Icon::Diff => "tree_icon_Diff",
            Icon::Sql => "tree_icon_Sql",
            Icon::Clojure => "tree_icon_Clojure",
            Icon::Edn => "tree_icon_Edn",
            Icon::Scala => "tree_icon_Scala",
            Icon::Go => "tree_icon_Go",
            Icon::Dart => "tree_icon_Dart",
            Icon::Firefox => "tree_icon_Firefox",
            Icon::Vs => "tree_icon_Vs",
            Icon::Perl => "tree_icon_Perl",
            Icon::Rss => "tree_icon_Rss",
            Icon::Fsharp => "tree_icon_Fsharp",
            Icon::Rust => "tree_icon_Rust",
            Icon::Dlang => "tree_icon_Dlang",
            Icon::Erlang => "tree_icon_Erlang",
            Icon::Elixir => "tree_icon_Elixir",
            Icon::Mix => "tree_icon_Mix",
            Icon::Vim => "tree_icon_Vim",
            Icon::Ai => "tree_icon_Ai",
            Icon::Psd => "tree_icon_Psd",
            Icon::Psb => "tree_icon_Psb",
            Icon::Typescript => "tree_icon_Typescript",
            Icon::Typescriptreact => "tree_icon_Typescriptreact",
            Icon::Julia => "tree_icon_Julia",
            Icon::Puppet => "tree_icon_Puppet",
            Icon::Vue => "tree_icon_Vue",
            Icon::Swift => "tree_icon_Swift",
            Icon::Gitconfig => "tree_icon_Gitconfig",
            Icon::Bashrc => "tree_icon_Bashrc",
            Icon::Favicon => "tree_icon_Favicon",
            Icon::Docker => "tree_icon_Docker",
            Icon::Gruntfile => "tree_icon_Gruntfile",
            Icon::Gulpfile => "tree_icon_Gulpfile",
            Icon::Dropbox => "tree_icon_Dropbox",
            Icon::License => "tree_icon_License",
            Icon::Procfile => "tree_icon_Procfile",
            Icon::Jquery => "tree_icon_Jquery",
            Icon::Angular => "tree_icon_Angular",
            Icon::Backbone => "tree_icon_Backbone",
            Icon::Requirejs => "tree_icon_Requirejs",
            Icon::Materialize => "tree_icon_Materialize",
            Icon::Mootools => "tree_icon_Mootools",
            Icon::Vagrant => "tree_icon_Vagrant",
            Icon::Svg => "tree_icon_Svg",
            Icon::Font => "tree_icon_Font",
            Icon::Text => "tree_icon_Text",
            Icon::Archive => "tree_icon_Archive",
            Icon::Unknown => "tree_icon_Unknonwn",
        }
    }
    pub fn as_glyph_and_color(&self) -> (&str, &str) {
        match *self {
            Icon::FolderClosed => ("", "#00afaf"),
            Icon::FolderOpened => ("", "#00afaf"),
            Icon::FolderSymlink => ("", "#00afaf"),
            Icon::File => ("", "#999999"),
            Icon::FileSymlink => ("", "#999999"),
            Icon::FileHidden => ("﬒", "#999999"),
            Icon::Excel => ("", "#207245"),
            Icon::Word => ("", "#185abd"),
            Icon::Ppt => ("", "#cb4a32"),
            Icon::Stylus => ("", "#8dc149"),
            Icon::Sass => ("", "#f55385"),
            Icon::Html => ("", "#e37933"),
            Icon::Xml => ("謹", "#e37933"),
            Icon::Ejs => ("", "#cbcb41"),
            Icon::Css => ("", "#519aba"),
            Icon::Webpack => ("ﰩ", "#519aba"),
            Icon::Markdown => ("", "#519aba"),
            Icon::Json => ("", "#cbcb41"),
            Icon::Javascript => ("", "#cbcb41"),
            Icon::Javascriptreact => ("", "#519aba"),
            Icon::Ruby => ("", "#cc3e44"),
            Icon::Php => ("", "#a074c4"),
            Icon::Python => ("", "#519aba"),
            Icon::Coffee => ("", "#cbcb41"),
            Icon::Mustache => ("", "#e37933"),
            Icon::Conf => ("", "#6d8086"),
            Icon::Image => ("", "#a074c4"),
            Icon::Ico => ("", "#cbcb41"),
            Icon::Twig => ("", "#8dc149"),
            Icon::C => ("", "#519aba"),
            Icon::H => ("", "#a074c4"),
            Icon::Haskell => ("", "#a074c4"),
            Icon::Lua => ("", "#519aba"),
            Icon::Java => ("", "#cc3e44"),
            Icon::Terminal => ("", "#4d5a5e"),
            Icon::Ml => ("λ", "#e37933"),
            Icon::Diff => ("", "#41535b"),
            Icon::Sql => ("", "#f55385"),
            Icon::Clojure => ("", "#8dc149"),
            Icon::Edn => ("", "#519aba"),
            Icon::Scala => ("", "#cc3e44"),
            Icon::Go => ("", "#519aba"),
            Icon::Dart => ("", "#03589C"),
            Icon::Firefox => ("", "#e37933"),
            Icon::Vs => ("", "#854CC7"),
            Icon::Perl => ("", "#519aba"),
            Icon::Rss => ("", "#FB9D3B"),
            Icon::Fsharp => ("", "#519aba"),
            Icon::Rust => ("", "#519aba"),
            Icon::Dlang => ("", "#cc3e44"),
            Icon::Erlang => ("", "#A90533"),
            Icon::Elixir => ("", "#a074c4"),
            Icon::Mix => ("", "#cc3e44"),
            Icon::Vim => ("", "#019833"),
            Icon::Ai => ("", "#cbcb41"),
            Icon::Psd => ("", "#519aba"),
            Icon::Psb => ("", "#519aba"),
            Icon::Typescript => ("", "#519aba"),
            Icon::Typescriptreact => ("", "#519aba"),
            Icon::Julia => ("", "#a074c4"),
            Icon::Puppet => ("", "#cbcb41"),
            Icon::Vue => ("﵂", "#8dc149"),
            Icon::Swift => ("", "#e37933"),
            Icon::Gitconfig => ("", "#41535b"),
            Icon::Bashrc => ("", "#4d5a5e"),
            Icon::Favicon => ("", "#cbcb41"),
            Icon::Docker => ("", "#519aba"),
            Icon::Gruntfile => ("", "#e37933"),
            Icon::Gulpfile => ("", "#cc3e44"),
            Icon::Dropbox => ("", "#0061FE"),
            Icon::License => ("", "#cbcb41"),
            Icon::Procfile => ("", "#a074c4"),
            Icon::Jquery => ("", "#1B75BB"),
            Icon::Angular => ("", "#E23237"),
            Icon::Backbone => ("", "#0071B5"),
            Icon::Requirejs => ("", "#F44A41"),
            Icon::Materialize => ("", "#EE6E73"),
            Icon::Mootools => ("", "#ECECEC"),
            Icon::Vagrant => ("", "#1563FF"),
            Icon::Svg => ("ﰟ", "#FFB13B"),
            Icon::Font => ("", "#999999"),
            Icon::Text => ("", "#999999"),
            Icon::Archive => ("", "#cc3e44"),
            Icon::Unknown => ("", "#999999"),
        }
    }
}

pub static GUI_COLORS: &[GuiColor] = &[
    GuiColor::BROWN,
    GuiColor::AQUA,
    GuiColor::BLUE,
    GuiColor::DARKBLUE,
    GuiColor::PURPLE,
    GuiColor::LIGHTPURPLE,
    GuiColor::RED,
    GuiColor::BEIGE,
    GuiColor::YELLOW,
    GuiColor::ORANGE,
    GuiColor::DARKORANGE,
    GuiColor::PINK,
    GuiColor::SALMON,
    GuiColor::GREEN,
    GuiColor::LIGHTGREEN,
    GuiColor::WHITE,
];

pub static ICONS: &[Icon] = &[
    Icon::FolderClosed,
    Icon::FolderOpened,
    Icon::FolderSymlink,
    Icon::File,
    Icon::FileSymlink,
    Icon::FileHidden,
    Icon::Excel,
    Icon::Word,
    Icon::Ppt,
    Icon::Stylus,
    Icon::Sass,
    Icon::Html,
    Icon::Xml,
    Icon::Ejs,
    Icon::Css,
    Icon::Webpack,
    Icon::Markdown,
    Icon::Json,
    Icon::Javascript,
    Icon::Javascriptreact,
    Icon::Ruby,
    Icon::Php,
    Icon::Python,
    Icon::Coffee,
    Icon::Mustache,
    Icon::Conf,
    Icon::Image,
    Icon::Ico,
    Icon::Twig,
    Icon::C,
    Icon::H,
    Icon::Haskell,
    Icon::Lua,
    Icon::Java,
    Icon::Terminal,
    Icon::Ml,
    Icon::Diff,
    Icon::Sql,
    Icon::Clojure,
    Icon::Edn,
    Icon::Scala,
    Icon::Go,
    Icon::Dart,
    Icon::Firefox,
    Icon::Vs,
    Icon::Perl,
    Icon::Rss,
    Icon::Fsharp,
    Icon::Rust,
    Icon::Dlang,
    Icon::Erlang,
    Icon::Elixir,
    Icon::Mix,
    Icon::Vim,
    Icon::Ai,
    Icon::Psd,
    Icon::Psb,
    Icon::Typescript,
    Icon::Typescriptreact,
    Icon::Julia,
    Icon::Puppet,
    Icon::Vue,
    Icon::Swift,
    Icon::Gitconfig,
    Icon::Bashrc,
    Icon::Favicon,
    Icon::Docker,
    Icon::Gruntfile,
    Icon::Gulpfile,
    Icon::Dropbox,
    Icon::License,
    Icon::Procfile,
    Icon::Jquery,
    Icon::Angular,
    Icon::Backbone,
    Icon::Requirejs,
    Icon::Materialize,
    Icon::Mootools,
    Icon::Vagrant,
    Icon::Svg,
    Icon::Font,
    Icon::Text,
    Icon::Archive,
    Icon::Unknown,
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

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum ColumnType {
    MARK,
    INDENT,
    GIT,
    ICON,
    FILENAME,
    SIZE,
    TIME,
}

impl From<&str> for ColumnType {
    fn from(s: &str) -> Self {
        match s {
            "mark" => ColumnType::MARK,
            "indent" => ColumnType::INDENT,
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

impl GuiColor {
    pub fn color_val(&self) -> &str {
        match *self {
            GuiColor::BROWN => "#905532",
            GuiColor::AQUA => "#3AFFDB",
            GuiColor::BLUE => "#689FB6",
            GuiColor::DARKBLUE => "#44788E",
            GuiColor::PURPLE => "#834F79",
            GuiColor::LIGHTPURPLE => "#834F79",
            GuiColor::RED => "#AE403F",
            GuiColor::BEIGE => "#F5C06F",
            GuiColor::YELLOW => "#F09F17",
            GuiColor::ORANGE => "#D4843E",
            GuiColor::DARKORANGE => "#F16529",
            GuiColor::PINK => "#CB6F6F",
            GuiColor::SALMON => "#EE6E73",
            GuiColor::GREEN => "#8FAA54",
            GuiColor::LIGHTGREEN => "#31B53E",
            GuiColor::WHITE => "#FFFFFF",
        }
    }

    pub fn hl_group_name(&self) -> &str {
        match *self {
            GuiColor::BROWN => "tree_color_brow",
            GuiColor::AQUA => "tree_color_aqua",
            GuiColor::BLUE => "tree_color_blue",
            GuiColor::DARKBLUE => "tree_color_darkblue",
            GuiColor::PURPLE => "tree_color_purple",
            GuiColor::LIGHTPURPLE => "tree_color_lightpurple",
            GuiColor::RED => "tree_color_red",
            GuiColor::BEIGE => "tree_color_beige",
            GuiColor::YELLOW => "tree_color_yellow",
            GuiColor::ORANGE => "tree_color_orange",
            GuiColor::DARKORANGE => "tree_color_darkorange",
            GuiColor::PINK => "tree_color_pink",
            GuiColor::SALMON => "tree_color_salmon",
            GuiColor::GREEN => "tree_color_green",
            GuiColor::LIGHTGREEN => "tree_color_lightgreen",
            GuiColor::WHITE => "tree_color_white",
        }
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

#[derive(Debug)]
pub struct FileItem {
    pub path: std::path::PathBuf,
    pub metadata: Metadata,
    pub level: isize,
    pub parent: Option<FileItemPtr>, // the index of the parent in the Tree::fileitems
    pub last: bool,
    pub id: usize,
    // pub git_map: HashMap<String, GitStatus>,
}
pub type FileItemPtr = std::sync::Arc<FileItem>;

impl FileItem {
    pub fn new(path: std::path::PathBuf, metadata: Metadata, id: usize) -> Self {
        Self {
            path,
            metadata,
            level: -1,
            parent: None,
            last: false,
            id,
        }
    }

    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(OsStr::to_str)
    }
}

#[derive(Debug)]
pub struct ColumnCell {
    pub col_start: usize,
    pub col_end: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub text: String,
    pub hl_group: Option<String>,
}

impl ColumnCell {
    pub fn new(tree: &Tree, fileitem: &FileItem, ty: ColumnType, is_root_cell: bool) -> Self {
        let mut text = String::new();
        let mut hl_group = None;
        match ty {
            ColumnType::MARK => {
                if fileitem.metadata.permissions().readonly() {
                    text = String::from(READ_ONLY_ICON);
                    hl_group = Some(String::from(GuiColor::BROWN.hl_group_name()))
                } else if tree.is_item_selected(fileitem.id) {
                    text = String::from(SELECTED_ICON);
                    hl_group = Some(String::from(GuiColor::GREEN.hl_group_name()))
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
                    let mut parent = &fileitem.parent;
                    while let Some(pf) = parent {
                        if i >= max_level {
                            break;
                        }
                        if pf.last {
                            inversed_elements.push("  ");
                        } else {
                            inversed_elements.push("│ ");
                        }
                        inversed_elements.push(prefix.as_str());
                        parent = &pf.parent;
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
                    text = String::new();
                    let dir_opened = match fileitem.path.to_str() {
                        Some(p) => tree.is_item_opened(p),
                        None => false,
                    };
                    if !is_root_cell {
                        let icon;
                        if dir_opened {
                            icon = Icon::FolderOpened;
                        } else if fileitem.metadata.file_type().is_symlink() {
                            icon = Icon::FolderSymlink;
                        } else {
                            icon = Icon::FolderClosed;
                        }
                        hl_group = Some(icon.hl_group_name().to_owned());
                        text.push_str(icon.as_glyph_and_color().0);
                    }
                } else {
                    let extension_icon = match fileitem.extension() {
                        Some(extension) => Icon::from(extension),
                        None => Icon::Unknown,
                    };
                    hl_group = Some(extension_icon.hl_group_name().to_owned());
                    text = extension_icon.as_glyph_and_color().0.to_owned();
                }
                text.push(' ');
            }
            ColumnType::FILENAME => {
                hl_group = Some(GuiColor::YELLOW.hl_group_name().to_owned());
                if is_root_cell {
                    text = tree.config.root_marker.clone();
                    text.push_str(fileitem.path.to_str().unwrap());
                } else {
                    text = String::from(fileitem.path.file_name().and_then(OsStr::to_str).unwrap());
                    if fileitem.metadata.is_dir() {
                        text.push('/');
                        hl_group = Some(String::from(GuiColor::BLUE.hl_group_name()));
                    }
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
            hl_group,
        }
    }
}
