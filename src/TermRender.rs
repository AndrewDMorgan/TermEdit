// snake case is just bad
#![allow(non_snake_case)]
#![allow(dead_code)]

// static color/mod pairs for default ascii/ansi codes
// colorCode (if any), mods, background (bool)   when called if background then add that color as background col
//      if no background found, provide no such parameter
// /033[ is the base with the ending post-fix being
// start;color;mod;mod;mod...suffix   how do i do different colored mods? Do i add another attachment? <- correct
// https://notes.burke.libbey.me/ansi-escape-codes/
// https://stackoverflow.com/questions/4842424/list-of-ansi-color-escape-sequences
// /033[... doesn't work; use /x1b[...
pub static CLEAR: &'static str = "\x1b[0m";

// *! color, modifiers, is background
pub static EMPTY_MODIFIER_REFERENCE: &[&str] = &[];

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

#[derive(Clone)]
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
    Default,

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
                UniqueColor::Dynamic((Some(format!("38;2;{};{};{}m", r, g, b)), EMPTY_MODIFIER_REFERENCE, false))
            },
            // background 24-bit? Make sure that's right
            ColorType::OnRGB (r, g, b) => {
                UniqueColor::Dynamic((Some(format!("48;2;{};{};{}m", r, g, b)), EMPTY_MODIFIER_REFERENCE, false))
            },
            ColorType::ANSI (index) => {
                UniqueColor::Dynamic((Some(format!("38;5;{}", index)), EMPTY_MODIFIER_REFERENCE, false))
            },
            ColorType::OnANSI (index) => {
                UniqueColor::Dynamic((Some(format!("48;5;{}", index)), EMPTY_MODIFIER_REFERENCE, false))
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

// Converts the given instance into the type Colored based on a provided
// set of modifiers.
pub trait Colorize {
    // adds a set of modifiers/colors
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored;

    // adds a single modifier/color
    fn Colorize (&self, colors: ColorType) -> Colored;
}

impl Colorize for &str {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypes(self, colors)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypes(self, vec![color])
    }
}

impl Colorize for String {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypes(self, colors)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypes(self, vec![color])
    }
}


// A colored string
// It stores all of its modifiers like colors/underlying/other
//#[derive(Clone)]
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Colored <'a> {
    text: &'a str,
    mods: Vec <String>,
    color: Option <String>,
    bgColor: Option <String>,
}

impl <'a> Colorize for Colored <'a> {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        let mut mods = vec![];
        for modifier in colors {
            mods.push(modifier);
        }
        Colored::GetFromColorTypes(self.text, mods)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypes(self.text, vec![color])
    }
}

impl <'a> Colored <'a> {
    pub fn new (text: &'a str) -> Colored <'a> {
        Colored {
            text,
            mods: vec![],
            color: None,
            bgColor: None,
        }
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
    pub fn GetFromColorTypes (text: &str, colors: Vec <ColorType>) -> Colored {
        let mut colored = Colored::new(text);
        for color in colors {
            colored.AddColor(color);
        } colored
    }

    // Takes a set of unique colors and generates a filled out instance
    pub fn GetFromUniqueColors (text: &str, uniqueColors: Vec <UniqueColor>) -> Colored {
        let mut colored = Colored::new(text);
        for color in uniqueColors {
            colored.AddUnique(color);
        } colored
    }
    
    pub fn GetText (&self) -> String {
        let mut text = String::new();
        if let Some(color) = &self.color {
            text.push_str(&format!(
                //
                "/x1b[{};{}m", color, self.mods.join(";")
            ));
        }
        if let Some(color) = &self.bgColor {
            text.push_str(&format!(
                //
                "/x1b[{}m", color  //, self.mods.join(";")  can't have modifiers on backgrounds?
            ));
        }
        text.push_str(self.text);
        text
    }
}

// A colored span of text (fancy string)
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Span <'a> {
    line: Vec <Colored <'a>>,
}

impl <'a> Span <'a> {
    fn Join (&self) -> String {
        let mut lastColored = vec![];
        let mut total = String::new();
        for colored in &self.line {
            if lastColored != colored.mods {
                lastColored = colored.mods.clone();
                total.push_str(CLEAR);
                total.push_str(&colored.mods.concat());
            } total.push_str(&colored.GetText());
        } total
    }
}


// Similar to a paragraph in Ratatui
// Windows are a block or section within the terminal space
// Multiple windows can be rendered at once
// Each window can contain its own text or logic
// This allows a separation/abstraction for individual sections
// This also allows for a cached window to be reused if temporarily closed
#[derive(Clone)]
pub struct Window <'a> {
    pub position: (u16, u16),
    pub size: (u16, u16),
    updated: Vec <bool>,

    // (Span, cached render)
    lines: Vec <(Span <'a>, String)>,

    bordered: bool,
    title: String,
    color: String,
}

impl <'a> Window <'a> {
    pub fn new (position: (u16, u16), size: (u16, u16)) -> Self {
        Window {
            position,
            size,
            updated: vec![],
            lines: vec![],
            bordered: false,
            title: String::new(),
            color: String::new(),  // format!("\x1b[38;2;{};{};{}m", 125, 125, 0),//String::new(),
        }
    }

    // Adds a border around the window/block
    pub fn Bordered (&mut self) {
        self.bordered = true;
    }

    // Sets/updates the title of the window/block
    pub fn Titled (&mut self, title: String) {
        self.title = title;
    }

    // Changes the size of the window
    pub fn Resize (&mut self, change: (isize, isize)) {
        self.size = (
            std::cmp::max(self.size.0 as isize + change.0, 0) as u16,
            std::cmp::max(self.size.1 as isize + change.1, 0) as u16
        );
    }

    // Updates the colorized rendering of the Spans for all lines
    // Each line is only re-computed if a change was indicated
    pub fn UpdateRender (&mut self) {
        for index in 0..self.updated.len() {
            if !self.updated[index] {  continue;  }
            self.updated[index] = false;
            self.lines[index].1 = self.lines[index].0.Join();
        }
    }
    
    // Gets the rendered text for the individual window
    // This shouldn't crash when rendering out of bounds unlike certain other libraries...
    pub fn GetRender (&self) -> Vec <String> {
        let mut text = vec![String::new()];
        
        // handling the top border
        let borderSize;
        if self.bordered {
            let mut lineSize = 1;
            text[0].push_str(&self.color);
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
            if index < self.lines.len() {
                let line = &self.lines[0..self.size.1 as usize - borderSize][0];
                lineText = &line.1[0..self.size.0 as usize - borderSize];
            } else {
                lineText = "";
            }

            // handling the side borders
            if self.bordered {
                text[index].push_str(&self.color);
                text[index].push('│');
                text[index].push_str(lineText);
                let padding = (self.size.0 as usize - 2) - lineText.len();
                text[index].push_str(&" ".repeat(padding));
                text[index].push('│');
                text[index].push_str(CLEAR);
            } else {
                text[index].push_str(lineText);
                let padding = (self.size.0 as usize) - lineText.len();
                text[index].push_str(&" ".repeat(padding));
            }
            text.push(String::new());
        }

        // handling the bottom border
        let lastIndex = text.len() - 1;
        if self.bordered {
            text[lastIndex].push_str(&self.color);
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
    pub fn UpdateLine (&mut self, index: usize, span: Span <'a>) {
        if index >= self.lines.len() {  return;  }
        self.lines[index] = (span, String::new());
        self.updated[index] = true;
    }

    // Appends a single line to the window
    pub fn AddLines (&mut self, span: Span <'a>) {
        self.lines.push((span, String::new()));
        self.updated.push(true);
    }

    // Takes a vector of type Span
    // That Span replaces the current set of lines for the window
    pub fn FromLines (&mut self, lines: Vec <Span <'a>>) {
        self.lines.clear(); self.updated.clear();
        for span in lines {
            self.lines.push((span, String::new()));
            self.updated.push(true);
        }
    }

    // checks to see if any lines need to be updated
    pub fn TryUpdateLines (&mut self, mut lines: Vec <Span <'a>>) {
        while let Some(span) = lines.pop() {
            let index = lines.len();  // the pop already subtracted one
            if span != self.lines[index].0 {
                self.lines[index] = (span, String::new());
                self.updated[index] = true;
            }
        }
    }
}


// the main window/application that handles all the windows
#[derive(Default, Clone)]
pub struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

// the main application. It stores and handles the active windows
// It also handles rendering the cumulative sum of the windows
#[derive(Default, Clone)]
pub struct App <'a> {
    area: Rect,
    activeWindows: Vec <Window <'a>>,
    windowReferences: std::collections::HashMap <String, usize>,
    changeWindowLayout: bool,
    updated: bool,
}

impl <'a> App <'a> {
    pub fn new () -> Self {
        let app = App {
            area: Rect {x: 0, y: 0, width: 5, height: 5},  // temp
            activeWindows: vec![],
            windowReferences: std::collections::HashMap::new(),
            changeWindowLayout: false,
            updated: true,
        };
        app
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
    pub fn AddWindow (&mut self, window: Window <'a>, name: String) {
        self.windowReferences.insert(name, self.windowReferences.len());
        self.activeWindows.push(window);
    }

    // Pops an active window.
    // Returns Ok(window) if the index is valid, or Err if out of bounds
    pub fn RemoveWindow (&mut self, name: String) -> Result <Window<'a>, String> {
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
        if self.changeWindowLayout {
            self.activeWindows.sort_by_key(|window| window.position.0);
        }

        // going through the sorted windows
        for window in &mut self.activeWindows {
            // if un-updated, this should only check for true in a vec a few times
            window.UpdateRender();

            let output = window.GetRender();
            for (index, line) in output.iter().enumerate() {
                let lineIndex = window.position.1 as usize + index;
                // finding the necessary padding
                let padding = window.position.0.saturating_sub(lineSizes[lineIndex]) as usize;
                finalLines[lineIndex].push_str(&" ".repeat(padding));

                // rendering the line of the window
                finalLines[lineIndex].push_str(&line);
                lineSizes[lineIndex] += window.size.0 + padding as u16;
            }
        }

        // printing the cumulative sum
        print!("{}", finalLines.join("\n"));
        self.updated = false;

        Ok(())
    }
}

