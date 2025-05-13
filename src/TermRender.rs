// snake case is just bad
#![allow(dead_code)]

use std::io::Write;

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
        Colored::GetFromColorTypesStr(&self.to_string(), colors)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypesStr(&self.to_string(), vec![color])
    }
}

impl Colorize for String {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypesStr(&self.clone(), colors)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypesStr(&self.clone(), vec![color])
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
        Colored::GetFromColorTypes(&self, mods)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypes(&self, vec![color])
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
    pub fn GetFromColorTypes (colored: &Colored, colors: Vec <ColorType>) -> Colored {
        let mut colored = Colored {
            text: colored.text.clone(),
            mods: colored.mods.clone(),
            color: colored.color.clone(),
            bgColor: colored.bgColor.clone(),
        };
        for color in colors {
            colored.AddColor(color);
        } colored
    }

    // Takes a set of color types and returns a filled out Colored instance
    pub fn GetFromColorTypesStr (text: &String, colors: Vec <ColorType>) -> Colored {
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

    pub fn GetText (&self, lastColor: &mut String) -> (String, usize) {
        let mut text = String::new();

        let col = match &self.color {
            Some(colr) => colr,
            _ => &String::new()
        };

        let bgCol = match &self.bgColor {
            Some(colr) => colr,
            _ => &String::new()
        };

        let color = match
            (self.bgColor.is_some(), self.color.is_some(), self.mods.is_empty())
        {
            (true, true, true) => format!("\x1b[0;{};{};{}m", col, bgCol, self.mods.join(";")),
            (true, true, false) => format!("\x1b[0;{};{}m", col, bgCol),
            (false, true, true) => format!("\x1b[0;{};{}m", col, self.mods.join(";")),
            (false, true, false) => format!("\x1b[0;{}m", col),
            (true, false, true) => format!("\x1b[0;{};{}m", bgCol, self.mods.join(";")),
            (true, false, false) => format!("\x1b[0;{}m", bgCol),
            (false, false, _) => String::from("\x1b[0m"),
        };

        if color != *lastColor {
            //text.push_str(CLEAR);
            text.push_str(&color);
            *lastColor = color;
        }

        text.push_str(&self.text);
        (text, self.text.len())

        /*let mut color = String::new();
        if self.mods.is_empty() && self.color.is_some() {
            color = format!("\x1b[{}m", *col);
        } else if self.color.is_some() {
            color = format!("\x1b[{};{}m", *col, self.mods.join(";"));
        }

        if let Some(bgCol) = &self.bgColor {
            color.push_str(&format!(
                "\x1b[{}m", bgCol  //, self.mods.join(";")  can't have modifiers on backgrounds?
            ));
        }*/
    }

    pub fn GetSize (&self) -> usize {
        self.text.len()
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

    pub fn Size (&self) -> usize {
        let mut size = 0;
        for colored in &self.line {
            size += colored.GetSize();
        }
        size
    }

    pub fn Join (&self) -> (String, usize) {
        //let mut lastColored = vec![];
        let mut lastColored = String::new();
        let mut total = String::new();
        let mut totalSize = 0;
        for colored in &self.line {
            /*if lastColored != colored.mods {
                lastColored = colored.mods.clone();
                total.push_str(CLEAR);
                total.push_str(&colored.mods.concat());
            }*/
            let (text, size) = colored.GetText(&mut lastColored);
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
    title: (Span, usize),
    color: Colored,
}

impl Window {
    pub fn new (position: (u16, u16), size: (u16, u16)) -> Self {
        Window {
            position,
            size,
            updated: vec![false; size.1 as usize],
            lines: vec![],
            bordered: false,
            title: (Span::default(), 0),
            color: Colored::new(String::new()),  // format!("\x1b[38;2;{};{};{}m", 125, 125, 0),//String::new(),
        }
    }

    pub fn Move (&mut self, newPosition: (u16, u16)) {
        if newPosition == self.position {  return;  }
        self.position = newPosition;
    }

    pub fn Colorizes (&mut self, colors: Vec <ColorType>) {
        for color in colors {
            self.color.AddColor(color);
        }
    }

    pub fn Colorize (&mut self, color: ColorType) {
        self.color.AddColor(color);
    }

    // Adds a border around the window/block
    pub fn Bordered (&mut self) {
        self.bordered = true;
    }

    // Sets/updates the title of the window/block
    pub fn Titled (&mut self, title: String) {
        self.title = (
            Span::FromTokens(
            vec![title.Colorizes(vec![])]),
            title.len()
        );
        //self.color.ChangeText(title);
    }

    pub fn HasTitle (&self) -> bool {
        self.title.1 != 0
    }

    pub fn TitledColored (&mut self, title: Span) {
        let size = title.Size();
        self.title = (title, size);
    }

    // Changes the size of the window
    pub fn Resize (&mut self, changed: (u16, u16)) {
        if self.size == changed {  return;  }
        self.size = (
            std::cmp::max(changed.0, 0),
            std::cmp::max(changed.1, 0)
        );
        self.updated = vec![false; self.size.1 as usize];
    }

    // Updates the colorized rendering of the Spans for all lines
    // Each line is only re-computed if a change was indicated
    /*pub fn UpdateRender (&mut self) {
        for index in 0..self.updated.len() {
            if !self.updated[index] {  continue;  }
            self.updated[index] = false;

            let (text, size) = self.lines[index].0.Join();
            self.lines[index].1 = text;
            self.lines[index].2 = size;
        }
    }*/

    // Clamps a string to a maximum length of visible UTF-8 characters while preserving escape codes
    fn ClampStringVisibleUTF_8 (text: &String, maxLength: usize) -> String {
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

    pub fn RenderWindowSlice (color: (String, usize),
                              bordered: bool,
                              renderText: (String, usize),
                              size: (u16, u16)
    ) -> String {
        let mut text = String::new();
        let lineSize;

        //let line = &self.lines[index - 1];//self.lines[0..self.size.1 as usize - borderSize][0];
        let borderSize = match bordered {
            true => 2, false => 0
        };
        let lineText = Window::ClampStringVisibleUTF_8(
            &renderText.0, size.0 as usize - borderSize
        );
        lineSize = std::cmp::min(renderText.1, size.0 as usize - borderSize);

        // handling the side borders
        if bordered {
            text.push_str(&color.0);
            text.push('│');
            text.push_str(CLEAR);
            text.push_str(&lineText);
            text.push_str(CLEAR);
            let padding = (size.0 as usize - 2) - lineSize;
            text.push_str(&" ".repeat(padding));
            text.push_str(&color.0);
            text.push('│');
            text.push_str(CLEAR);
        } else {
            text.push_str(&lineText);
            let padding = (size.0 as usize) - lineSize;
            text.push_str(&" ".repeat(padding));
        }
        text
    }

    pub fn GetRenderClosure (&mut self) -> Vec <(Box <dyn FnOnce () -> String + Send>, u16, u16)> {
        // these will need to be sorted by row, and the cursor movement is handled externally (the u16 pair)
        let mut renderClosures: Vec <(Box <dyn FnOnce () -> String + Send>, u16, u16)> = vec![];
        let borderColor = self.color.GetText(&mut String::new());

        // make sure to not call UpdateRender when using closures
        let borderedSize = {
            if self.bordered {  1  }
            else {  0  }
        };
        let mut updated = false;
        for index in borderedSize..self.size.1 as usize - borderedSize {
            if self.updated[index] {  continue;  }
            updated = true;
            self.updated[index] = true;

            let (text, size);
            if index - borderedSize < self.lines.len() {
                (text, size) = self.lines[index - borderedSize].0.Join();
                self.lines[index - borderedSize].1 = text.clone();
                self.lines[index - borderedSize].2 = size.clone();
            } else {
                (text, size) = (String::new(), 0);
            }

            // creating the closure
            let color = borderColor.clone();
            let windowSize = self.size;  // idk a better way to do this other than cloning
            let bordered = self.bordered;

            let closure = move || {
                let slice = Window::RenderWindowSlice(color, bordered, (text, size), windowSize);
                slice
            };
            renderClosures.push((Box::new(closure), self.position.0, self.position.1 + index as u16));
        }

        if updated && self.bordered {
            self.updated[0] = true;
            self.updated[self.size.1 as usize - 1] = true;

            // adding the top and bottom lines to the closures
            let color = borderColor.clone();
            let windowSize = self.size.0;  // idk a better way to do this other than cloning
            let closure = move || {  // top
                let mut text = String::new();
                text.push_str(&color.0);
                text.push('└');
                text.push_str(&"─".repeat(windowSize as usize - 2));
                text.push('┘');
                text.push_str(CLEAR);
                text
            };
            renderClosures.push((Box::new(closure), self.position.0, self.position.1 + self.size.1 - 1));

            // bottom
            let color = borderColor;  // consuming border color here
            let windowSize = self.size.0;  // idk a better way to do this other than cloning
            let closure = move || {
                let mut text = String::new();
                text.push_str(&color.0);
                text.push('┌');
                text.push_str(&"─".repeat(windowSize as usize - 2));
                text.push('┐');
                text.push_str(CLEAR);
                text
            };
            renderClosures.push((Box::new(closure), self.position.0, self.position.1));
        }

        renderClosures
    }

    // Gets the rendered text for the individual window
    // This shouldn't crash when rendering out of bounds unlike certain other libraries...
    pub fn GetRender (&self) -> Vec <String> {
        let mut text = vec![String::new()];
        let color = self.color.GetText(&mut String::new());

        // handling the top border
        let borderSize;
        if self.bordered {
            let mut lineSize = 1;
            text[0].push_str(&color.0);
            text[0].push('┌');
            let splitSize = (self.size.0 - 2) / 2 - self.title.1 as u16 / 2;
            lineSize += splitSize;
            text[0].push_str(&"─".repeat(splitSize as usize));
            lineSize += self.title.1 as u16;
            text[0].push_str(&self.title.0.Join().0);
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
                lineText = Window::ClampStringVisibleUTF_8(
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
        self.updated[index] = false;
    }

    // Appends a single line to the window
    pub fn AddLine (&mut self, span: Span) {
        self.lines.push((span, String::new(), 0));
        self.updated.push(false);
    }

    // Takes a vector of type Span
    // That Span replaces the current set of lines for the window
    pub fn FromLines (&mut self, lines: Vec <Span>) {
        self.lines.clear();// self.updated.clear();
        let mut index = {
            if self.bordered {  1  }
            else {  0  }
        };
        for span in lines {
            self.lines.push((span, String::new(), 0));
            self.updated[index] = false;
            index += 1;
        }
    }

    // checks to see if any lines need to be updated
    pub fn TryUpdateLines (&mut self, mut lines: Vec <Span>) {
        if lines.len() != self.lines.len() {
            let mut index = 0;
            for span in lines {
                if index >= self.updated.len() {  break;  }
                self.lines.push((span, String::new(), 0));
                self.updated[index] = false;
                index += 1;
            }
            return;
        }
        let mut index = lines.len();
        while let Some(span) = lines.pop() {
            index -= 1;  // the pop already subtracted one
            if span != self.lines[index].0 {
                self.lines[index] = (span, String::new(), 0);
                self.updated[index] = false;
            }
        }
    }

    pub fn IsEmpty (&self) -> bool {
        self.lines.is_empty()
    }

    pub fn UpdateAll (&mut self) {
        for line in self.updated.iter_mut() {
            *line = false;
        }
    }
}


// the main window/application that handles all the windows
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

// the main application. It stores and handles the active windows
// It also handles rendering the cumulative sum of the windows
#[derive(Debug, Default)]
pub struct App {
    area: Rect,
    activeWindows: Vec <(Window, Vec <String>)>,  // window, mods
    windowReferences: std::collections::HashMap <String, usize>,
    changeWindowLayout: bool,
    updated: bool,
    renderHandle: Option <std::thread::JoinHandle <()>>,
    buffer: std::sync::Arc <parking_lot::RwLock <String>>,
    resetWindows: bool,
}

impl App {
    pub fn new () -> Self {
        print!("\x1B[2J\x1B[H");  // clearing the screen
        App {
            area: Rect::default(),
            activeWindows: vec![],
            windowReferences: std::collections::HashMap::new(),
            changeWindowLayout: true,
            updated: true,
            renderHandle: None,
            buffer: std::sync::Arc::new(parking_lot::RwLock::new(String::new())),
            resetWindows: false,
        }
    }

    pub fn ContainsWindow (&self, name: String) -> bool {
        self.windowReferences.contains_key(&name)
    }

    pub fn GetWindowReference (&self, name: String) -> &Window {
        &self.activeWindows[self.windowReferences[&name]].0
    }

    pub fn GetWindowReferenceMut (&mut self, name: String) -> &mut Window {
        //self.updated = true;  // assuming something is being changed
        &mut self.activeWindows[self.windowReferences[&name]].0
    }

    pub fn UpdateWindowLayoutOrder (&mut self) {
        self.changeWindowLayout = true;
        //self.updated = true;
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
    pub fn AddWindow (&mut self, window: Window, name: String, keywords: Vec <String>) {
        self.changeWindowLayout = true;
        self.windowReferences.insert(name, self.windowReferences.len());
        self.activeWindows.push((window, keywords));
        //self.updated = true;
    }

    // Pops an active window.
    // Returns Ok(window) if the index is valid, or Err if out of bounds
    pub fn RemoveWindow (&mut self, name: String) -> Result <Window, String> {
        self.changeWindowLayout = true;
        self.resetWindows = true;
        //self.updated = true;

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

        Ok(self.activeWindows.remove(index).0)
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
    pub fn Render (&mut self) {
        //let start = std::time::Instant::now();
        if self.renderHandle.is_some() {
            let handle = self.renderHandle.take().unwrap();
            let _ = handle.join();
        }

        let size = self.GetTerminalSize().unwrap();

        self.buffer.write().clear();
        if size.0 != self.area.width || size.1 != self.area.height || self.resetWindows {
            self.resetWindows = false;
            *self.buffer.write() = String::with_capacity((size.0 * size.1 * 3) as usize);

            // making sure the windows get updated
            //self.updated = true;
            for window in &mut self.activeWindows {
                window.0.UpdateAll();
            }
            print!("\x1B[2J\x1B[H");  // re-clearing the screen (everything will need to update....)
        }

        self.area = Rect {
            x: 0,
            y: 0,
            width: size.0,
            height: size.1,
        };

        // only re-rendering on updates (otherwise the current results are perfectly fine)
        // this should reduce CPU usage by a fair bit and allow a fast refresh rate if needed
        let mut updated = false;
        for window in &self.activeWindows {
            if !window.0.updated.contains(&false) {  continue;  }
            updated = true;
            break;
        }
        if !updated {  return;  }  // Ok(());  }

        // sorting the windows based on the horizontal position
        let mut referenceArray = vec![];
        for keyPair in &self.windowReferences {
            referenceArray.push(keyPair.1);
        }

        if self.changeWindowLayout {
            referenceArray.sort_by_key( |index| self.activeWindows[**index].0.position.0 );
            self.changeWindowLayout = false;
        }

        // stores the draw calls
        let mut drawCalls: Vec <(Box <dyn FnOnce () -> String + Send>, u16, u16)> = vec![];

        // going through the sorted windows
        for index in referenceArray {
            let window = &mut self.activeWindows[*index];
            drawCalls.append(&mut window.0.GetRenderClosure());
        }

        let size = (self.area.width, self.area.height);
        let buffer = self.buffer.clone();
        //println!("Num calls: {}", drawCalls.len());
        self.renderHandle = Some(std::thread::spawn(move || {
            // the buffer for the render string

            // sorting the calls by action row (and left to right for same row calls)
            drawCalls.sort_by_key(|drawCall| drawCall.2 * size.0 + drawCall.1);

            // iterating through the calls (consuming drawCalls)
            let writeBuffer = &mut *buffer.write();
            for call in drawCalls {
                // moving the cursor into position
                // ESC[{line};{column}H
                writeBuffer.push_str("\x1b[");
                App::PushU16(writeBuffer, call.2);
                writeBuffer.push_str(";");
                App::PushU16(writeBuffer, call.1);
                writeBuffer.push_str("H");

                let output = call.0();
                writeBuffer.push_str(&output);
            }

            // moving the cursor to the bottom right
            writeBuffer.push_str("\x1b[");
            App::PushU16(writeBuffer, size.1);
            writeBuffer.push_str(";");
            App::PushU16(writeBuffer, size.0);
            writeBuffer.push_str("H ");

            // rendering the buffer
            let mut out = std::io::stdout().lock();
            out.write_all(writeBuffer.as_bytes()).unwrap();
            out.flush().unwrap();
        }));

        //let elapsed = start.elapsed();
        //panic!("Render thread completed in {:?}", elapsed);
    }

    pub fn PushU16 (buffer: &mut String, mut value: u16) {
        let mut reserved = [0u32; 5];
        let mut i = 0;
        //println!(": {}", value);
        loop {
            reserved[i] = (value % 10) as u32;
            if value < 10 {  break;  }
            value /= 10;
            i += 1;
        }
        //println!("[{}, {}; {:?}]", value, i, reserved);
        for index in (0..=i).rev() {
            //println!("({:?}, {})", char::from_digit(reserved[index], 10), reserved[index]);
            buffer.push(char::from_digit(reserved[index], 10).unwrap());
        }
    }

    pub fn GetWindowNames (&self) -> Vec<&String> {
        let mut names = vec![];
        for name in  self.windowReferences.keys() {
            names.push(name);
        } names
    }

    /// Prunes all windows which contain one of the specified keywords
    /// Returns the number of windows pruned
    pub fn PruneByKeywords (&mut self, keywords: Vec <String>) -> usize {
        let mut pruned = vec![];
        for (index, window) in self.activeWindows.iter().enumerate() {
            for word in &window.1 {
                if keywords.contains(word) {
                    //println!("\n:{:?}::", (index, word));
                    pruned.push(index);
                    break;
                }
            }
        }
        if pruned.is_empty() {  return 0;  }
        self.changeWindowLayout = true;
        self.resetWindows = true;
        self.updated = true;

        let mut numPruned = 0;
        let mut iter = pruned.iter();
        // pruned should be in ascending order
        while let Some(index) = iter.next() {
            self.PruneUpdate(*index, &mut numPruned);
        } numPruned
    }

    fn PruneUpdate (&mut self, index: usize, numPruned: &mut usize) {
        // shifting all the indexes
        let mut toRemove = vec![];
        for pair in self.windowReferences.iter_mut() {
            if *pair.1 == index-*numPruned {
                toRemove.push(pair.0.clone());
                continue;
            }
            if *pair.1 >= index-*numPruned {  *pair.1 -= 1  }
        }
        for key in toRemove {
            self.windowReferences.remove(&key);
        }
        let _ = self.activeWindows.remove(index-*numPruned);
        *numPruned += 1;
    }

    /// Prunes all windows based on a given key (closure).
    /// Returns the number of windows pruned.
    /// If the closure returns true, the element is pruned. If it returns false it's kept.
    pub fn PruneByKey (&mut self, key: Box <dyn Fn (&Vec <String>) -> bool>) -> usize {
        let mut pruned = vec![];
        for (index, window) in self.activeWindows.iter().enumerate() {
            if key(&window.1) {
                pruned.push(index);
            }
        }
        if pruned.is_empty() {  return 0;  }
        self.changeWindowLayout = true;
        self.resetWindows = true;
        self.updated = true;

        let mut numPruned = 0;
        let mut iter = pruned.iter();
        // pruned should be in ascending order
        while let Some(index) = iter.next() {
            self.PruneUpdate(*index, &mut numPruned);
        } numPruned
    }

    /// Gets the names to all windows which contain at least one of the
    /// specified keywords.
    pub fn GetWindowsByKeywords (&self, keywords: Vec <String>) -> Vec <&String> {
        let mut names = vec![];
        for name in &self.windowReferences {
            for keyword in &self.activeWindows[*name.1].1 {
                if keywords.contains(keyword) {
                    names.push(name.0);
                    break;
                }
            }
        }
        names
    }

    pub fn GetWindowsByKeywordsNonRef (&self, keywords: Vec <String>) -> Vec <String> {
        let mut names = vec![];
        for name in &self.windowReferences {
            for keyword in &self.activeWindows[*name.1].1 {
                if keywords.contains(keyword) {
                    names.push(name.0.clone());
                    break;
                }
            }
        }
        names
    }

    /// Gets the names to all windows which satisfy the given key (closure).
    /// If the closure returns true, the name is provided. Otherwise, it's
    /// considered unrelated.
    pub fn GetWindowsByKey (&self, key: Box <dyn Fn (&Vec <String>) -> bool>) -> Vec <&String> {
        let mut names = vec![];
        for name in &self.windowReferences {
            if key(&self.activeWindows[*name.1].1) {
                names.push(name.0);
            }
        }
        names
    }

    /// Checks if a given window contains a specific keyword.
    pub fn WindowContainsKeyword (&self, windowName: &String, keyword: &String) -> bool {
        let windowIndex = self.windowReferences[windowName];
        self.activeWindows[windowIndex].1.contains(keyword)
    }

    pub fn ChangedWindowLayout (&self) -> bool {
        self.changeWindowLayout
    }
}

