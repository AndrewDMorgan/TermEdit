// snake case is just bad
#![allow(dead_code)]

//* Add a check for updated in the GetRender method for windows
//* Make a proc macro for easier colorizing (Color![White, Dim, ...])
//      -- expands to something like .Colorizes(vec![ColorType::White, ...])
//      -- right now it's just very wordy (a bit annoying to type bc/ of that)


// static color/mod pairs for default ascii/ansi codes
// colorCode (if any), mods, background (bool)   when called if background then add that color as background col
//      if no background found, provide no such parameter
// /033[ is the base with the ending post-fix being
// start;color;mod;mod;mod...suffix   how do I do different colored mods? Do I add another attachment? <- correct
// https://notes.burke.libbey.me/ansi-escape-codes/
// https://stackoverflow.com/questions/4842424/list-of-ansi-color-escape-sequences
// /033[... doesn't work; use /x1b[...
pub static CLEAR: &'static str = "\x1b[0m";

// * color, modifiers, is_background
pub static EMPTY_MODIFIER_REFERENCE: &[&str] = &[];  // making a default static type is annoying

pub static BLACK:      (Option <&str>, &[&str], bool) = (Some("30"), &[], false);
pub static RED:        (Option <&str>, &[&str], bool) = (Some("31"), &[], false);
pub static GREEN:      (Option <&str>, &[&str], bool) = (Some("32"), &[], false);
pub static YELLOW:     (Option <&str>, &[&str], bool) = (Some("33"), &[], false);
pub static BLUE:       (Option <&str>, &[&str], bool) = (Some("34"), &[], false);
pub static MAGENTA:    (Option <&str>, &[&str], bool) = (Some("35"), &[], false);
pub static CYAN:       (Option <&str>, &[&str], bool) = (Some("36"), &[], false);
pub static WHITE:      (Option <&str>, &[&str], bool) = (Some("37"), &[], false);
pub static DEFAULT:    (Option <&str>, &[&str], bool) = (Some("39"), &[], false);

pub static BRIGHT_BLACK:   (Option <&str>, &[&str], bool) = (Some("90"), &[], true );
pub static BRIGHT_RED:     (Option <&str>, &[&str], bool) = (Some("91"), &[], true );
pub static BRIGHT_GREEN:   (Option <&str>, &[&str], bool) = (Some("92"), &[], true );
pub static BRIGHT_YELLOW:  (Option <&str>, &[&str], bool) = (Some("93"), &[], true );
pub static BRIGHT_BLUE:    (Option <&str>, &[&str], bool) = (Some("94"), &[], true );
pub static BRIGHT_MAGENTA: (Option <&str>, &[&str], bool) = (Some("95"), &[], true );
pub static BRIGHT_CYAN:    (Option <&str>, &[&str], bool) = (Some("96"), &[], true );
pub static BRIGHT_WHITE:   (Option <&str>, &[&str], bool) = (Some("97"), &[], true );
pub static BRIGHT_DEFAULT: (Option <&str>, &[&str], bool) = (Some("99"), &[], true );

pub static ON_BLACK:   (Option <&str>, &[&str], bool) = (Some("40"), &[], true );
pub static ON_RED:     (Option <&str>, &[&str], bool) = (Some("41"), &[], true );
pub static ON_GREEN:   (Option <&str>, &[&str], bool) = (Some("42"), &[], true );
pub static ON_YELLOW:  (Option <&str>, &[&str], bool) = (Some("43"), &[], true );
pub static ON_BLUE:    (Option <&str>, &[&str], bool) = (Some("44"), &[], true );
pub static ON_MAGENTA: (Option <&str>, &[&str], bool) = (Some("45"), &[], true );
pub static ON_CYAN:    (Option <&str>, &[&str], bool) = (Some("46"), &[], true );
pub static ON_WHITE:   (Option <&str>, &[&str], bool) = (Some("47"), &[], true );
pub static ON_DEFAULT: (Option <&str>, &[&str], bool) = (Some("49"), &[], true );

pub static ON_BRIGHT_BLACK:   (Option <&str>, &[&str], bool) = (Some("100"), &[], true );
pub static ON_BRIGHT_RED:     (Option <&str>, &[&str], bool) = (Some("101"), &[], true );
pub static ON_BRIGHT_GREEN:   (Option <&str>, &[&str], bool) = (Some("102"), &[], true );
pub static ON_BRIGHT_YELLOW:  (Option <&str>, &[&str], bool) = (Some("103"), &[], true );
pub static ON_BRIGHT_BLUE:    (Option <&str>, &[&str], bool) = (Some("104"), &[], true );
pub static ON_BRIGHT_MAGENTA: (Option <&str>, &[&str], bool) = (Some("105"), &[], true );
pub static ON_BRIGHT_CYAN:    (Option <&str>, &[&str], bool) = (Some("106"), &[], true );
pub static ON_BRIGHT_WHITE:   (Option <&str>, &[&str], bool) = (Some("107"), &[], true );
pub static ON_BRIGHT_DEFAULT: (Option <&str>, &[&str], bool) = (Some("109"), &[], true );

pub static BOLD:      (Option <&str>, &[&str], bool) = (None    , &["1"], false);
pub static DIM:       (Option <&str>, &[&str], bool) = (None    , &["2"], false);
pub static ITALIC:    (Option <&str>, &[&str], bool) = (None    , &["3"], false);
pub static UNDERLINE: (Option <&str>, &[&str], bool) = (None    , &["4"], false);
pub static BLINK:     (Option <&str>, &[&str], bool) = (None    , &["5"], false);
pub static REVERSE:   (Option <&str>, &[&str], bool) = (None    , &["7"], false);
pub static HIDE:      (Option <&str>, &[&str], bool) = (None    , &["8"], false);

#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
// Different base ascii text modifiers (static constants)
pub enum ColorType {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    #[default] Default,

    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    BrightDefault,

    OnBlack,
    OnRed,
    OnGreen,
    OnYellow,
    OnBlue,
    OnMagenta,
    OnCyan,
    OnWhite,
    OnDefault,

    OnBrightBlack,
    OnBrightRed,
    OnBrightGreen,
    OnBrightYellow,
    OnBrightBlue,
    OnBrightMagenta,
    OnBrightCyan,
    OnBrightWhite,
    OnBrightDefault,

    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hide,

    OnRGB (u8, u8, u8),
    RGB (u8, u8, u8),
    OnANSI (u8),
    ANSI (u8),
}

// Stores a unique color type.
// The unique types are either a fully static color
// or a partially dynamic type.
// This allows for a passing of different types,
// circumventing lifetime issues while preserving statics.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum UniqueColor {
    Static  ((Option <&'static str>, &'static [&'static str], bool)),
    Dynamic ((Option <   String   >, &'static [&'static str], bool)),
}

impl UniqueColor {
    // Converts a static slice into a vector of type String
    fn IntoStringVec (&self, attributes: &'static [&'static str]) -> Vec<String> {
        let mut mods = vec![];
        for modifier in attributes {
            mods.push(modifier.to_string())
        }
        mods
    }

    // Converts the unique color into a standardized tuple form.
    // In other words, converts the dynamic and static versions
    // into a single unified version for easier handling
    pub fn UnwrapIntoTuple (&self) -> (Option <String>, Vec <String>, bool) {
        match self {
            UniqueColor::Static(s) => {
                (match &s.0 {
                    Some(t) => Some(t.to_string()),
                    None => None,
                }, self.IntoStringVec(s.1), s.2)
            },
            UniqueColor::Dynamic(s) => {
                (s.0.clone(), self.IntoStringVec(s.1), s.2)
            },
        }
    }
}

impl ColorType {
    // Converts the color type into a unique color (static or dynamic)
    pub fn GetColor (&self) -> UniqueColor {
        match self {
            ColorType::Black => { UniqueColor::Static(BLACK) },
            ColorType::Red => { UniqueColor::Static(RED) },
            ColorType::Green => { UniqueColor::Static(GREEN) },
            ColorType::Yellow => { UniqueColor::Static(YELLOW) },
            ColorType::Blue => { UniqueColor::Static(BLUE) },
            ColorType::Magenta => { UniqueColor::Static(MAGENTA) },
            ColorType::Cyan => { UniqueColor::Static(CYAN) },
            ColorType::White => { UniqueColor::Static(WHITE) },
            ColorType::Default => { UniqueColor::Static(DEFAULT) },

            ColorType::BrightBlack => { UniqueColor::Static(BRIGHT_BLACK) },
            ColorType::BrightRed => { UniqueColor::Static(BRIGHT_RED) },
            ColorType::BrightGreen => { UniqueColor::Static(BRIGHT_GREEN) },
            ColorType::BrightYellow => { UniqueColor::Static(BRIGHT_YELLOW) },
            ColorType::BrightBlue => { UniqueColor::Static(BRIGHT_BLUE) },
            ColorType::BrightMagenta => { UniqueColor::Static(BRIGHT_MAGENTA) },
            ColorType::BrightCyan => { UniqueColor::Static(BRIGHT_CYAN) },
            ColorType::BrightWhite => { UniqueColor::Static(BRIGHT_WHITE) },
            ColorType::BrightDefault => { UniqueColor::Static(BRIGHT_DEFAULT) },

            ColorType::OnBlack => { UniqueColor::Static(ON_BRIGHT_BLACK) },
            ColorType::OnRed => { UniqueColor::Static(ON_BRIGHT_RED) },
            ColorType::OnGreen => { UniqueColor::Static(ON_BRIGHT_GREEN) },
            ColorType::OnYellow => { UniqueColor::Static(ON_BRIGHT_YELLOW) },
            ColorType::OnBlue => { UniqueColor::Static(ON_BRIGHT_BLUE) },
            ColorType::OnMagenta => { UniqueColor::Static(ON_BRIGHT_MAGENTA) },
            ColorType::OnCyan => { UniqueColor::Static(ON_BRIGHT_CYAN) },
            ColorType::OnWhite => { UniqueColor::Static(ON_BRIGHT_WHITE) },
            ColorType::OnDefault => { UniqueColor::Static(ON_DEFAULT) },

            ColorType::OnBrightBlack => { UniqueColor::Static(ON_BLACK) },
            ColorType::OnBrightRed => { UniqueColor::Static(ON_RED) },
            ColorType::OnBrightGreen => { UniqueColor::Static(ON_GREEN) },
            ColorType::OnBrightYellow => { UniqueColor::Static(ON_YELLOW) },
            ColorType::OnBrightBlue => { UniqueColor::Static(ON_BLUE) },
            ColorType::OnBrightMagenta => { UniqueColor::Static(ON_MAGENTA) },
            ColorType::OnBrightCyan => { UniqueColor::Static(ON_CYAN) },
            ColorType::OnBrightWhite => { UniqueColor::Static(ON_WHITE) },
            ColorType::OnBrightDefault => { UniqueColor::Static(ON_BRIGHT_DEFAULT) },

            // 24-bit? I think so but make sure it works
            ColorType::RGB (r, g, b) => {
                UniqueColor::Dynamic((Some(format!("38;2;{};{};{}", r, g, b)), EMPTY_MODIFIER_REFERENCE, false))
            },
            // background 24-bit? Make sure that's right
            ColorType::OnRGB (r, g, b) => {
                UniqueColor::Dynamic((Some(format!("48;2;{};{};{}", r, g, b)), EMPTY_MODIFIER_REFERENCE, true))
            },
            ColorType::ANSI (index) => {
                UniqueColor::Dynamic((Some(format!("38;5;{}", index)), EMPTY_MODIFIER_REFERENCE, false))
            },
            ColorType::OnANSI (index) => {
                UniqueColor::Dynamic((Some(format!("48;5;{}", index)), EMPTY_MODIFIER_REFERENCE, true))
            },

            ColorType::Bold => { UniqueColor::Static(BOLD) },
            ColorType::Dim => { UniqueColor::Static(DIM) },
            ColorType::Italic => { UniqueColor::Static(ITALIC) },
            ColorType::Underline => { UniqueColor::Static(UNDERLINE) },
            ColorType::Blink => { UniqueColor::Static(BLINK) },
            ColorType::Reverse => { UniqueColor::Static(REVERSE) },
            ColorType::Hide => { UniqueColor::Static(HIDE) },
        }
    }
}

// Color setters for standard primitives

/// Converts the given instance into the type Colored based on a provided
/// set of modifiers (in the form of ColorType).
pub trait Colorize {
    // adds a set of modifiers/colors
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored;

    // adds a single modifier/color
    fn Colorize (&self, colors: ColorType) -> Colored;
}

impl Colorize for &str {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypes(&self.to_string(), colors)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypes(&self.to_string(), vec![color])
    }
}

impl Colorize for String {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypes(&self.clone(), colors)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypes(&self.clone(), vec![color])
    }
}


// A colored string
// It stores all of its modifiers like colors/underlying/other
//#[derive(Clone)]
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Colored {
    text: String,
    mods: Vec <String>,
    color: Option <String>,
    bgColor: Option <String>,
}

impl Colorize for Colored {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        let mut mods = vec![];
        for modifier in colors {
            mods.push(modifier);
        }
        Colored::GetFromColorTypes(&self.text, mods)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypes(&self.text, vec![color])
    }
}

impl Colored {
    pub fn new (text: String) -> Colored {
        Colored {
            text,
            mods: vec![],
            color: None,
            bgColor: None,
        }
    }

    pub fn ChangeText (&mut self, text: String) {
        self.text = text;
    }

    // Adds a color type
    pub fn AddColor (&mut self, color: ColorType) {
        self.AddUnique(color.GetColor());

    }

    // Adds a unique color
    pub fn AddUnique (&mut self, uniqueColor: UniqueColor) {
        let (color, mods, background) = uniqueColor.UnwrapIntoTuple();
        if background {  self.bgColor = color;  }
        else if let Some(col) = color {
            // making sure to not overwrite the existing color if this is None
            self.color = Some(col);
        }
        for modifier in mods {
            self.mods.push(modifier);
        }
    }

    // Takes a set of color types and returns a filled out Colored instance
    pub fn GetFromColorTypes (text: &String, colors: Vec <ColorType>) -> Colored {
        let mut colored = Colored::new(text.clone());
        for color in colors {
            colored.AddColor(color);
        } colored
    }

    // Takes a set of unique colors and generates a filled out instance
    pub fn GetFromUniqueColors (text: String, uniqueColors: Vec <UniqueColor>) -> Colored {
        let mut colored = Colored::new(text);
        for color in uniqueColors {
            colored.AddUnique(color);
        } colored
    }

    pub fn GetText (&self) -> (String, usize) {
        let mut text = String::new();
        if let Some(color) = &self.color {
            text.push_str(&format!(
                //
                //":{}:{}:", color, self.mods.join(";")
                "\x1b[0;{};{}m", color, self.mods.join(";")
            ));
        }
        if let Some(color) = &self.bgColor {
            text.push_str(&format!(
                "\x1b[{}m", color  //, self.mods.join(";")  can't have modifiers on backgrounds?
            ));
        }
        text.push_str(&self.text);
        (text, self.text.len())
    }
}

// A colored span of text (fancy string)
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Span {
    line: Vec <Colored>,
}

impl Span {
    pub fn FromTokens (tokens: Vec <Colored>) -> Self {
        Span {
            line: tokens,
        }
    }

    fn Join (&self) -> (String, usize) {
        //let mut lastColored = vec![];
        let mut total = String::new();
        let mut totalSize = 0;
        for colored in &self.line {
            /*if lastColored != colored.mods {
                lastColored = colored.mods.clone();
                total.push_str(CLEAR);
                total.push_str(&colored.mods.concat());
            }*/
            let (text, size) = colored.GetText();
            total.push_str(&text);
            totalSize += size;
        }
        (total, totalSize)
    }
}


// Similar to a paragraph in Ratatui
// Windows are a block or section within the terminal space
// Multiple windows can be rendered at once
// Each window can contain its own text or logic
// This allows a separation/abstraction for individual sections
// This also allows for a cached window to be reused if temporarily closed
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Window {
    pub position: (u16, u16),
    pub size: (u16, u16),
    updated: Vec <bool>,

    // (Span, cached render, num visible chars)
    lines: Vec <(Span, String, usize)>,

    bordered: bool,
    title: String,
    color: Colored,
}

impl Window {
    pub fn new (position: (u16, u16), size: (u16, u16)) -> Self {
        Window {
            position,
            size,
            updated: vec![],
            lines: vec![],
            bordered: false,
            title: String::new(),
            color: Colored::new(String::new()),  // format!("\x1b[38;2;{};{};{}m", 125, 125, 0),//String::new(),
        }
    }

    pub fn Move (&mut self, newPosition: (u16, u16)) {
        self.position = newPosition;
    }

    pub fn Colorizes (&mut self, colors: Vec <ColorType>) {
        for color in colors {
            self.color.AddColor(color);
        }
    }

    pub fn Colorize <'b> (&mut self, color: ColorType) {
        self.color.AddColor(color);
    }

    // Adds a border around the window/block
    pub fn Bordered (&mut self) {
        self.bordered = true;
    }

    // Sets/updates the title of the window/block
    pub fn Titled (&mut self, title: String) {
        self.title = title;
        //self.color.ChangeText(title);
    }

    // Changes the size of the window
    pub fn Resize (&mut self, change: (u16, u16)) {
        self.size = (
            std::cmp::max(self.size.0 + change.0, 0),
            std::cmp::max(self.size.1 + change.1, 0)
        );
    }

    // Updates the colorized rendering of the Spans for all lines
    // Each line is only re-computed if a change was indicated
    pub fn UpdateRender (&mut self) {
        for index in 0..self.updated.len() {
            if !self.updated[index] {  continue;  }
            self.updated[index] = false;

            let (text, size) = self.lines[index].0.Join();
            self.lines[index].1 = text;
            self.lines[index].2 = size;
        }
    }

    // Clamps a string to a maximum length of visible UTF-8 characters while preserving escape codes
    fn ClampStringVisibleUTF_8 (&self, text: &String, maxLength: usize) -> String {
        let mut accumulative: String = String::new();

        let mut visible = 0;
        let mut inEscape = false;
        let mut chars = text.chars();
        while let Some(chr) = chars.next() {
            if chr == '\x1b' {
                inEscape = true;
            } else if inEscape {
                if chr == 'm' {
                    inEscape = false;
                }
            } else {
                visible += 1;
                if visible > maxLength {  break;  }
            }
            accumulative.push_str(&chr.to_string());
        }

        accumulative
    }

    // Gets the rendered text for the individual window
    // This shouldn't crash when rendering out of bounds unlike certain other libraries...
    pub fn GetRender (&self) -> Vec <String> {
        let mut text = vec![String::new()];
        let color = self.color.GetText();

        // handling the top border
        let borderSize;
        if self.bordered {
            let mut lineSize = 1;
            text[0].push_str(&color.0);
            text[0].push('┌');
            let splitSize = (self.size.0 - 2) / 2 - self.title.len() as u16 / 2;
            lineSize += splitSize;
            text[0].push_str(&"─".repeat(splitSize as usize));
            lineSize += self.title.len() as u16;
            text[0].push_str(&self.title);
            //let lineSize = text[0].len();
            text[0].push_str(&"─".repeat(
                (self.size.0 as usize).saturating_sub(1 + lineSize as usize)
            ));
            text[0].push('┐');
            text[0].push_str(CLEAR);
            //text[0].push('\n');  // fix this
            text.push(String::new());
            borderSize = 2;
        }
        else {  borderSize = 0;  }
        let bordered = borderSize / 2;
        for index in bordered..self.size.1 as usize - bordered {
            let lineText;
            let lineSize;
            if index <= self.lines.len() {
                let line = &self.lines[index - 1];//self.lines[0..self.size.1 as usize - borderSize][0];
                /*lineText = &line.1[0..std::cmp::min(
                    self.size.0 as usize - borderSize,
                    line.1.len()
                )];*/
                /*lineText = line.1.get(0..std::cmp::min(
                    self.size.0 as usize - borderSize,
                    line.1.len()
                )).unwrap_or("");*/  // the clamping won't work with how it's being done rn bc/ of multi-byte chars
                lineText = self.ClampStringVisibleUTF_8(
                    &line.1, self.size.0 as usize - borderSize
                );
                lineSize = std::cmp::min(self.lines[index - 1].2, self.size.0 as usize - borderSize);
            } else {
                lineText = String::new();
                lineSize = 0;
            }

            // handling the side borders
            if self.bordered {
                text[index].push_str(&color.0);
                text[index].push('│');
                text[index].push_str(CLEAR);
                text[index].push_str(&lineText);
                text[index].push_str(CLEAR);
                let padding = (self.size.0 as usize - 2) - lineSize;
                text[index].push_str(&" ".repeat(padding));
                text[index].push_str(&color.0);
                text[index].push('│');
                text[index].push_str(CLEAR);
            } else {
                text[index].push_str(&lineText);
                let padding = (self.size.0 as usize) - lineSize;
                text[index].push_str(&" ".repeat(padding));
            }
            text.push(String::new());
        }

        // handling the bottom border
        let lastIndex = text.len() - 1;
        if self.bordered {
            text[lastIndex].push_str(&color.0);
            text[lastIndex].push('└');
            text[lastIndex].push_str(&"─".repeat(self.size.0 as usize - 2));
            text[lastIndex].push('┘');
            text[lastIndex].push_str(CLEAR);
        } else {
            // removing the last \n
            text.pop();
        }
        text
    }

    // Replaces a single line with an updated version
    pub fn UpdateLine (&mut self, index: usize, span: Span) {
        if index >= self.lines.len() {  return;  }
        self.lines[index] = (span, String::new(), 0);
        self.updated[index] = true;
    }

    // Appends a single line to the window
    pub fn AddLine (&mut self, span: Span) {
        self.lines.push((span, String::new(), 0));
        self.updated.push(true);
    }

    // Takes a vector of type Span
    // That Span replaces the current set of lines for the window
    pub fn FromLines (&mut self, lines: Vec <Span>) {
        self.lines.clear(); self.updated.clear();
        for span in lines {
            self.lines.push((span, String::new(), 0));
            self.updated.push(true);
        }
    }

    // checks to see if any lines need to be updated
    pub fn TryUpdateLines (&mut self, mut lines: Vec <Span>) {
        if lines.len() != self.lines.len() {
            while let Some(span) = lines.pop() {
                self.lines.push((span, String::new(), 0));
            }
            return;
        }
        while let Some(span) = lines.pop() {
            let index = lines.len();  // the pop already subtracted one
            if span != self.lines[index].0 {
                self.lines[index] = (span, String::new(), 0);
                self.updated[index] = true;
            }
        }
    }

    pub fn IsEmpty (&self) -> bool {
        self.lines.is_empty()
    }
}


// the main window/application that handles all the windows
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

// the main application. It stores and handles the active windows
// It also handles rendering the cumulative sum of the windows
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct App {
    area: Rect,
    activeWindows: Vec <Window>,
    windowReferences: std::collections::HashMap <String, usize>,
    changeWindowLayout: bool,
    updated: bool,
}

impl App {
    pub fn new () -> Self {
        App {
            area: Rect::default(),
            activeWindows: vec![],
            windowReferences: std::collections::HashMap::new(),
            changeWindowLayout: true,
            updated: true,
        }
    }

    pub fn ContainsWindow (&self, name: String) -> bool {
        self.windowReferences.contains_key(&name)
    }

    pub fn GetWindowReference (&self, name: String) -> &Window {
        &self.activeWindows[self.windowReferences[&name]]
    }

    pub fn GetWindowReferenceMut (&mut self, name: String) -> &mut Window {
        self.updated = true;  // assuming something is being changed
        &mut self.activeWindows[self.windowReferences[&name]]
    }

    pub fn UpdateWindowLayoutOrder (&mut self) {
        self.changeWindowLayout = true;
        self.updated = true;
    }

    pub fn GetTerminalSize (&self) -> Result <(u16, u16), std::io::Error> {
        crossterm::terminal::size()
    }

    // Gets the current window size and position
    // This returns a reference to this instance
    pub fn GetWindowArea (&self) -> &Rect {
        &self.area
    }

    // Adds a new active window
    pub fn AddWindow (&mut self, window: Window, name: String) {
        self.changeWindowLayout = true;
        self.windowReferences.insert(name, self.windowReferences.len());
        self.activeWindows.push(window);
        self.updated = true;
    }

    // Pops an active window.
    // Returns Ok(window) if the index is valid, or Err if out of bounds
    pub fn RemoveWindow (&mut self, name: String) -> Result <Window, String> {
        self.changeWindowLayout = true;
        self.updated = true;

        if !self.windowReferences.contains_key(&name) {
            return Err(format!("No window named '{}' found", name));
        }
        let index = *self.windowReferences.get(&name).unwrap();
        if index >= self.activeWindows.len() {
            return Err(format!(
                "Invalid index; Accessed at {}, but the size is {}",
                index, self.activeWindows.len()
            ));
        }

        // updating the references list
        self.windowReferences.remove(&name);
        let mut keysToModify = vec![];
        for key in self.windowReferences.keys() {
            if self.windowReferences.get(key).unwrap() > &index {
                keysToModify.push(key.clone());
            }
        }
        for key in keysToModify {
            *self.windowReferences.get_mut(&key).unwrap() -= 1;
        }

        Ok(self.activeWindows.remove(index))
    }

    // Gets a range of visible UTF-8 characters while preserving escape codes
    pub fn GetSliceUTF_8 (text: &String, range: std::ops::Range <usize>) -> String
    where
        std::ops::Range<usize>: Iterator<Item = usize>
    {
        let mut visible = 0;
        let mut inEscape = false;
        let mut slice = String::new();
        let mut textChars = text.chars();
        while let Some(chr) = textChars.next() {
            if chr == '\x1b' {
                inEscape = true;

                // making sure to keep the initial escape codes
                slice.push_str(&chr.to_string());
            } else if inEscape {
                inEscape = chr != 'm';

                // making sure to keep the initial escape codes
                slice.push_str(&chr.to_string());
            } else {
                visible += 1;
                if visible >= range.start {
                    if visible < range.end {
                        // adding the element to the slice
                        slice.push_str(&chr.to_string());
                        continue;
                    }
                    return slice;  // no need to continue
                }
            }
        } slice
    }

    // Renders all the active windows to the consol
    // It also clears the screen from previous writing
    pub fn Render (&mut self) -> Result <(), std::io::Error> {
        // only re-rendering on updates (otherwise the current results are perfectly fine)
        // this should reduce CPU usage by a fair bit and allow a fast refresh rate if needed
        if !self.updated {  return Ok(());  }

        let size = self.GetTerminalSize()?;
        self.area = Rect {
            x: 0,
            y: 0,
            width: size.0,
            height: size.1,
        };

        let mut finalLines = vec![String::new(); self.area.height as usize + 1];
        let mut lineSizes = vec![0; self.area.height as usize + 1];

        // sorting the windows based on the horizontal position
        let mut referenceArray = vec![];
        for keyPair in &self.windowReferences {
            referenceArray.push(keyPair.1);
        }

        if self.changeWindowLayout {
            referenceArray.sort_by_key( |index| self.activeWindows[**index].position.0 );
            self.changeWindowLayout = false;
        }

        // going through the sorted windows
        for index in referenceArray {
            let window = &mut self.activeWindows[*index];
            // if un-updated, this should only check for true in a vec a few times
            window.UpdateRender();

            let output = window.GetRender();
            for (index, line) in output.iter().enumerate() {
                let lineIndex = window.position.1 as usize + index;
                // finding the necessary padding
                // figure out intersections...
                let padding = window.position.0.saturating_sub(lineSizes[lineIndex]) as usize;
                finalLines[lineIndex].push_str(&" ".repeat(padding));

                // rendering the line of the window
                if window.position.0 + window.size.0 > lineSizes[lineIndex] {
                    finalLines[lineIndex].push_str(
                        // adjust this to count for non-visible characters...
                        // once this is adjusted, I think it should work for any type of intersection
                        // future elements are always at a deeper depth (first rendered is always on top; too lazy to change)
                        &App::GetSliceUTF_8(&line,
                                            lineSizes[lineIndex]
                                                .saturating_sub(window.position.0
                                                    .saturating_sub(1)
                                                ) as usize
                                            ..line.len()
                        )
                    );
                }
                lineSizes[lineIndex] += window.size.0 + padding as u16;
            }
        }

        // printing the cumulative sum
        print!("{}", finalLines.join("\n"));
        self.updated = false;

        Ok(())
    }
}

