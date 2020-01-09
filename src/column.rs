use crate::tree::Tree;
use std::fs::Metadata;
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

#[derive(PartialEq, Eq)]
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
    pub path: String,
    pub metadata: Metadata,
    pub level: usize,
    pub opened_tree: bool,
    pub selected: bool,
    pub parent: Option<usize>, // the index of the parent in the tree list
    pub last: bool,
    // pub git_map: HashMap<String, GitStatus>,
}

impl FileItem {
    pub fn new(path: String, metadata: Metadata) -> Self {
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
}

pub struct Cell {
    col_start: usize,
    col_end: usize,
    byte_start: usize,
    byte_end: usize,
    // TODO: size is not equal to the column num
    text: Vec<u8>,
    color: GuiColor,
}

impl Cell {
    pub fn new(tree: &Tree, fileitem: &FileItem, ty: ColumnType) {
        let mut text;
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
            _ => {}
        };
    }
}
