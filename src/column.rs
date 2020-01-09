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
