// snake case is just bad
#![allow(non_snake_case)]
#![allow(dead_code)]

// todo!! Check if there's a better way to represent multiple colors and modifiers

// colors & constants
static CLEAR: &'static str = "";
pub static BLACK: &'static str = "";
pub static RED: &'static str = "";
pub static GREEN: &'static str = "";
pub static YELLOW: &'static str = "";
pub static BLUE: &'static str = "";
pub static MAGENTA: &'static str = "";
pub static CYAN: &'static str = "";
pub static WHITE: &'static str = "";

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
}

impl ColorType {
    pub fn GetColor (&self) -> &str {
        match self {
            ColorType::Black => { BLACK },
            ColorType::Red => { RED },
            ColorType::Green => { GREEN },
            ColorType::Yellow => { YELLOW },
            ColorType::Blue => { BLUE },
            ColorType::Magenta => { MAGENTA },
            ColorType::Cyan => { CYAN },
            ColorType::White => { WHITE },
        }
    }
}


// Color setters for standard primitives

// Converts the given instance into the type Colored based on a provided
// set of modifiers.
pub trait Colorize {
    fn Colorize (&self, colors: Vec <ColorType>) -> Colored;
}

impl Colorize for &str {
    fn Colorize (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypes(self, colors)
    }
}

impl Colorize for String {
    fn Colorize (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypes(&self, colors)
    }
}


// A colored string
// It stores all of its modifiers like colors/underlying/other
//#[derive(Clone)]
#[derive(Clone)]
pub struct Colored <'a> {
    text: &'a str,
    mods: Vec <String>,
}

impl <'a> Colorize for Colored <'a> {
    fn Colorize (&self, colors: Vec <ColorType>) -> Colored {
        let mut mods = vec![];
        for modifier in colors {
            mods.push(modifier);
        }
        Colored::GetFromColorTypes(self.text, mods)
    }
}

impl <'a> Colored <'a> {
    pub fn AddColor (&mut self, color: ColorType) {
        self.mods.push(color.GetColor().to_string());
    }
    
    pub fn GetFromColorTypes (text: &str, colors: Vec <ColorType>) -> Colored {
        let mut mods = vec![];

        for color in &colors {
            mods.push(color.GetColor().to_string().clone());
        }
        Colored {
            text,
            mods,
        }
    }
    
    pub fn GetText (&self) -> String {
        let mut text = self.mods.concat();
        text.push_str(self.text);
        text
    }
}

// A colored span of text (fancy string)
#[derive(Clone)]
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
    pub position: (usize, usize),
    pub size: (usize, usize),
    updated: Vec <bool>,
    lines: Vec <(Span <'a>, String)>,
    bordered: bool,
    title: String,
}

impl <'a> Window <'a> {
    fn new (position: (usize, usize), size: (usize, usize)) -> Self {
        Window {
            position,
            size,
            updated: vec![],
            lines: vec![],
            bordered: false,
            title: String::new(),
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
            std::cmp::max(self.size.0 as isize + change.0, 0) as usize,
            std::cmp::max(self.size.1 as isize + change.1, 0) as usize
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
            text[0].push('┌');
            let splitSize = (self.size.0 - 2 ) / 2;
            text[0].push_str(&"─".repeat(splitSize));
            text[0].push_str(&self.title);
            let lineSize = text[0].len();
            text[0].push_str(&"─".repeat(
                self.size.0 - 1 - lineSize
            ));
            text[0].push('┐');
            text[0].push('\n');
            borderSize = 2;
        }
        else {  borderSize = 0;  }
        for (index, line) in self.lines[0..self.size.1 - borderSize].iter().enumerate() {
            let lineText = &line.1[0..self.size.0 - borderSize];

            // handling the side borders
            if self.bordered {
                text[index].push('|');
                text[index].push_str(lineText);
                let padding = (self.size.0 - 2) - lineText.len();
                text[index].push_str(&" ".repeat(padding));
                text[index].push('|');
            } else {
                text[index].push_str(lineText);
            }
            text.push(String::new());
            //text.push('\n');  // ┘ └ ─ ┌ ┐
        }

        // handling the bottom border
        let lastIndex = text.len();
        if self.bordered {
            text[lastIndex].push('└');
            text[lastIndex].push_str(&"─".repeat(self.size.0 - 2));
            text[lastIndex].push('┘');
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
}


// the main window/application that handles all the windows
#[derive(Default, Clone)]
pub struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

// the main application. It stores and handles the active windows
// It also handles rendering the cumulative sum of the windows
#[derive(Default, Clone)]
pub struct App <'a> {
    area: Rect,
    activeWindows: Vec <Window <'a>>,
    changeWindowLayout: bool,
    updated: bool,
}

impl <'a> App <'a> {
    pub fn new () -> Self {
        App {
            area: Rect {x: 0, y: 0, width: 5, height: 5},  // temp
            activeWindows: vec![],
            changeWindowLayout: false,
            updated: true,
        }
    }

    // Gets the current window size and position
    // This returns a reference to this instance
    pub fn GetWindowArea (&self) -> &Rect {
        &self.area
    }

    // Adds a new active window
    pub fn AddWindow (&mut self, window: Window <'a>) {
        self.activeWindows.push(window);
    }

    // Pops an active window.
    // Returns Ok(window) if the index is valid, or Err if out of bounds
    pub fn RemoveWindow (&mut self, index: usize) -> Result <Window<'a>, String> {
        if index >= self.activeWindows.len() {
            return Err(format!(
                "Invalid index; Accessed at {}, but the size is {}",
                index, self.activeWindows.len()
            ));
        }
        Ok(self.activeWindows.remove(index))
    }
    
    // Renders all the active windows to the consol
    // It also clears the screen from previous writing
    pub fn Render (&mut self) {
        // only re-rendering on updates (otherwise the current results are perfectly fine)
        // this should reduce CPU usage by a fair bit and allow a fast refresh rate if needed
        if !self.updated {  return;  }

        let mut finalLines = vec![String::new(); self.area.height];

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
                let lineIndex = window.position.1 + index;
                // finding the necessary padding
                let padding = window.position.0.saturating_sub(finalLines[lineIndex].len());
                finalLines[lineIndex].push_str(&" ".repeat(padding));

                // rendering the line of the window
                finalLines[lineIndex].push_str(&line);
            }
        }

        // printing the cumulative sum
        print!("{}", finalLines.join("\n"));
        self.updated = false;
    }
}

