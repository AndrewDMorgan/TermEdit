// snake case is just bad
#![allow(non_snake_case)]

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use vte::{Parser, Perform};

use crossterm::terminal::enable_raw_mode;
use arboard::Clipboard;

mod CodeTabs;
mod Tokens;

use CodeTabs::*;
use Tokens::*;


use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};


#[derive(Debug, Default)]
pub enum FileTabs {
    Outline,
    #[default] Files,
}

#[derive(Debug, Default)]
pub struct FileBrowser {
    files: Vec <String>,  // stores the names

    fileTab: FileTabs,
    fileCursor: usize,
    outlineCursor: usize,
}

impl FileBrowser {
    pub fn LoadFilePath (&mut self, pathInput: &str, codeTabs: &mut CodeTabs::CodeTabs) {
        self.files.clear();
        codeTabs.tabs.clear();
        if let Ok(paths) = std::fs::read_dir(pathInput) {
            for path in paths.flatten() {
                if std::fs::FileType::is_file(&path.file_type().unwrap()) {
                    let name = path.file_name().to_str().unwrap_or("").to_string();
                    self.files.push(name.clone());
                    
                    // loading the file's contents
                    let mut lines: Vec <String> = vec!();
                    
                    let mut fullPath = pathInput.to_string();
                    fullPath.push_str(&name);

                    let msg = fullPath.as_str().trim();  // temporary for debugging
                    let contents = std::fs::read_to_string(&fullPath).expect(msg);
                    let mut current = String::new();
                    for chr in contents.chars() {
                        if chr == '\n' {
                            lines.push(current.clone());
                            current.clear();
                        } else {
                            current.push(chr);
                        }
                    }
                    lines.push(current);

                    let mut tab = CodeTab {
                        lines,
                        ..Default::default()
                    };

                    tab.fileName = fullPath;

                    tab.lineTokens.clear();
                    for line in tab.lines.iter() {
                        tab.lineTokens.push(
                            {
                                let ending = tab.fileName.split('.').last().unwrap_or("");
                                GenerateTokens(line.clone(), ending)
                            }
                        );
                    }
                    (tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens);

                    codeTabs.tabs.push(tab);
                    codeTabs.tabFileNames.push(name.clone());
                }
            }
        }
    }

    pub fn MoveCursorDown (&mut self, outline: &[Vec<usize>], _rootNode: &ScopeNode) {
        if matches!(self.fileTab, FileTabs::Outline) {
            self.outlineCursor = std::cmp::min(
                self.outlineCursor + 1,
                outline.len() - 1
            );
        } else {
            // todo
        }
    }
    pub fn MoveCursorUp (&mut self) {
        if matches!(self.fileTab, FileTabs::Outline) {
            self.outlineCursor = self.outlineCursor.saturating_sub(1);  // simple
        } else {
            // todo
        }
    }
}


#[derive(Debug, Default)]
pub enum TabState {
    #[default] Code,
    Files,
    Tabs,
}

#[derive(Debug, Default)]
pub enum AppState {
    #[default] Tabs,
    CommandPrompt,
}

#[derive(PartialEq)]
pub enum KeyModifiers {
    Shift,
    Command,
    Option,
    Control,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum KeyCode {
    Delete,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Return,
    Escape,
}

pub enum MouseEventType {
    Null,
    Left,
    Right,
    Middle,
    Down,
    Up,
}

pub enum MouseState {
    Release,
    Press,
    Hold,
    Null,
}

pub struct MouseEvent {
    eventType: MouseEventType,
    position: (u16, u16),
    state: MouseState,
}

#[derive(Default)]
pub struct KeyParser {
    keyModifiers: Vec <KeyModifiers>,
    keyEvents: std::collections::HashMap <KeyCode, bool>,
    charEvents: Vec <char>,
    inEscapeSeq: bool,
    bytes: usize,
    mouseEvent: Option <MouseEvent>,
    mouseModifiers: Vec <KeyModifiers>,
}

impl KeyParser {
    pub fn new () -> Self {
        KeyParser {
            keyEvents: std::collections::HashMap::from([
                (KeyCode::Delete, false),
                (KeyCode::Tab, false),
                (KeyCode::Left, false),
                (KeyCode::Right, false),
                (KeyCode::Up, false),
                (KeyCode::Down, false),
                (KeyCode::Return, false),
                (KeyCode::Escape, false),
            ]),
            keyModifiers: vec!(),
            charEvents: vec!(),
            inEscapeSeq: false,
            bytes: 0,
            mouseEvent: None,
            mouseModifiers: vec!(),
        }
    }

    pub fn ClearEvents (&mut self) {
        self.charEvents.clear();
        self.keyModifiers.clear();
        self.mouseModifiers.clear();
        self.keyEvents.clear();
        self.inEscapeSeq = false;

        if let Some(event) = &mut self.mouseEvent {
            match event.state {
                MouseState::Press => {
                    event.state = MouseState::Hold;
                },
                MouseState::Hold if matches!(event.eventType, MouseEventType::Down | MouseEventType::Up) => {
                    event.state = MouseState::Release;
                },
                MouseState::Release => {
                    event.state = MouseState::Null;
                    event.eventType = MouseEventType::Null;
                },
                MouseState::Hold => {
                },
                _ => {},
            }
        }
    }

    pub fn ContainsChar (&self, chr: char) -> bool {
        self.charEvents.contains(&chr)
    }

    pub fn ContainsModifier (&self, modifier: KeyModifiers) -> bool {
        self.keyModifiers.contains(&modifier)
    }

    pub fn ContainsMouseModifier (&self, modifier: KeyModifiers) -> bool {
        self.mouseModifiers.contains(&modifier)
    }

    pub fn ContainsKeyCode (&self, key: KeyCode) -> bool {
        *self.keyEvents.get(&key).unwrap_or(&false)
    }

}

async fn enableMouseCapture() {
    let mut stdout = tokio::io::stdout();
    let _ = stdout.write_all(b"echo -e \"\x1B[?1006h").await;
    let _ = stdout.write_all(b"\x1B[?1000h").await; // Enable basic mouse mode
    let _ = stdout.write_all(b"\x1B[?1003h").await; // Enable all motion events
    std::mem::drop(stdout);
}

async fn disableMouseCapture() {
    let mut stdout = tokio::io::stdout();
    let _ = stdout.write_all(b"\x1B[?1000l").await; // Disable mouse mode
    let _ = stdout.write_all(b"\x1B[?1003l").await; // Disable motion events
    std::mem::drop(stdout);
}

impl Perform for KeyParser {
    fn execute(&mut self, byte: u8) {
        match byte {
            0x1B => {
                self.inEscapeSeq = true;
            },
            0x0D => {  // return aka \n
                self.keyEvents.insert(KeyCode::Return, true);
            },
            0x09 => {
                self.keyEvents.insert(KeyCode::Tab, true);
            },
            _ => {},
        }
        //println!("byte {}: '{}'", byte, byte as char);
    }
    
    fn print(&mut self, chr: char) {
        if self.inEscapeSeq || self.bytes > 1 {  return;  }

        if chr as u8 == 0x7F {
            self.keyEvents.insert(KeyCode::Delete, true);
            return;
        }
        if !(chr.is_ascii_graphic() || chr.is_whitespace()) {  return;  }
        //println!("char {}: '{}'", chr as u8, chr);
        self.charEvents.push(chr);
    }
    
    fn csi_dispatch(&mut self, params: &vte::Params, _: &[u8], _: bool, c: char) {
        self.inEscapeSeq = false;  // resetting the escape sequence

        let numbers: Vec<u16> = params.iter().map(|p| p[0]).collect();

        // mouse handling
        if c == 'M' || c == 'm' {
            if let Some([byte, x, y]) = numbers.get(0..3) {
                let button = byte & 0b11; // Mask lowest 2 bits (button type)
                //println!("button: {}, numbers: {:?}", button, numbers);

                // adding key press modifiers
                if (byte & 32) != 0 {
                    self.keyModifiers.push(KeyModifiers::Shift);
                } if (byte & 64) != 0 {
                    self.keyModifiers.push(KeyModifiers::Option);
                } if (byte & 128) != 0 {
                    self.keyModifiers.push(KeyModifiers::Control);
                }

                //println!("Code: {:?} / {}", numbers, c);

                let isScroll = (byte & 64) != 0;
                let eventType = match (isScroll, button) {
                    (true, 0) => MouseEventType::Up,   // 1???? ig so
                    (true, 1) => MouseEventType::Down, // 2???? ig so
                    (false, 0) => MouseEventType::Left,
                    (false, 1) => MouseEventType::Middle,
                    (false, 2) => MouseEventType::Right,
                    _ => MouseEventType::Null
                };

                if matches!(eventType, MouseEventType::Left) && numbers[0] == 4 {
                    self.mouseModifiers.push(KeyModifiers::Shift);
                }

                if let Some(event) = &mut self.mouseEvent {
                    if matches!(eventType, MouseEventType::Left) &&
                        event.position != (*x, *y) &&
                        matches!(event.state, MouseState::Hold) &&
                        c == 'M'
                    {
                        event.position = (*x, *y);
                        return;
                    }
                }

                self.mouseEvent = Some(MouseEvent {
                    eventType,
                    position: (*x, *y),
                    state: {
                        match c {
                            'M' => MouseState::Press,
                            'm' => MouseState::Release,
                            _ => MouseState::Null,
                        }
                    },
                });

            }

            return;
        }

        //for number in &numbers {println!("{}", number);}
        if c == '~' {  // this section is for custom escape codes
            if numbers == [3, 2] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 3] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Option);
            } else if numbers == [3, 4] {
                self.keyEvents.insert(KeyCode::Left, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 5] {
                self.keyEvents.insert(KeyCode::Right, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 6] {
                self.keyEvents.insert(KeyCode::Up, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 7] {
                self.keyEvents.insert(KeyCode::Down, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 8] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Option);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 9] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Command);
            } else if numbers == [3, 10] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 11] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('s');  // command + s
            } else if numbers == [3, 12] {  // lrud
                self.keyEvents.insert(KeyCode::Left, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 13] {
                self.keyEvents.insert(KeyCode::Right, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 14] {
                self.keyEvents.insert(KeyCode::Up, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 15] {
                self.keyEvents.insert(KeyCode::Down, true);
                self.keyModifiers.push(KeyModifiers::Command);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 16] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('c');
            } else if numbers == [3, 17] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('v');
            } else if numbers == [3, 18] {
                self.keyModifiers.push(KeyModifiers::Command);
                self.charEvents.push('x');
            }
        } else {  // this checks existing escape codes of 1 parameter/ending code (they don't end with ~)
            match c as u8 {
                0x5A => {
                    self.keyEvents.insert(KeyCode::Tab, true);
                    self.keyModifiers.push(KeyModifiers::Shift);
                },
                0x44 => {
                    self.keyEvents.insert(KeyCode::Left, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    } else if numbers == [1, 2] {
                        self.keyModifiers.push(KeyModifiers::Shift);
                    } else if numbers == [1, 4] {
                        self.keyModifiers.push(KeyModifiers::Option);
                        self.keyModifiers.push(KeyModifiers::Shift);
                    }
                },
                0x43 => {
                    self.keyEvents.insert(KeyCode::Right, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    } else if numbers == [1, 2] {
                        self.keyModifiers.push(KeyModifiers::Shift);
                    } else if numbers == [1, 4] {
                        self.keyModifiers.push(KeyModifiers::Option);
                        self.keyModifiers.push(KeyModifiers::Shift);
                    }
                },
                0x41 => {
                    self.keyEvents.insert(KeyCode::Up, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    } else if numbers == [1, 2] {
                        self.keyModifiers.push(KeyModifiers::Shift);
                    } else if numbers == [1, 4] {
                        self.keyModifiers.push(KeyModifiers::Option);
                        self.keyModifiers.push(KeyModifiers::Shift);
                    }
                },
                0x42 => {
                    self.keyEvents.insert(KeyCode::Down, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    } else if numbers == [1, 2] {
                        self.keyModifiers.push(KeyModifiers::Shift);
                    } else if numbers == [1, 4] {
                        self.keyModifiers.push(KeyModifiers::Option);
                        self.keyModifiers.push(KeyModifiers::Shift);
                    }
                },
                _ => {},
            }
        }
    }
}


#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    appState: AppState,
    tabState: TabState,
    codeTabs: CodeTabs::CodeTabs,
    currentCommand: String,
    fileBrowser: FileBrowser,
    area: Rect,
    lastScrolled: u128,

    debugInfo: String,
}

impl App {

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        enable_raw_mode()?; // Enable raw mode for direct input handling

        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        
        let mut clipboard = Clipboard::new().unwrap();

        self.fileBrowser.LoadFilePath("src/", &mut self.codeTabs);
        self.fileBrowser.fileCursor = 1;
        self.codeTabs.currentTab = 1;

        let mut parser = Parser::new();
        let mut keyParser = KeyParser::new();
        let mut buffer = [0; 128];//[0; 10];
        let mut stdin = tokio::io::stdin();
        
        while !self.exit {
            if self.exit {
                break;
            }

            buffer.fill(0);
            
            tokio::select! {
                result = stdin.read(&mut buffer) => {
                    if let Ok(n) = result {
                        keyParser.bytes = n;
                        
                        if n == 1 && buffer[0] == 0x1B {
                            keyParser.keyEvents.insert(KeyCode::Escape, true);
                        } else {
                            parser.advance(&mut keyParser, &buffer[..n]);
                        }
                    }
                    if self.exit {
                        break;
                    }
                },
                _ = tokio::time::sleep(std::time::Duration::from_nanos(0)) => {
                    terminal.draw(|frame| self.draw(frame))?;
                    if self.exit {
                        break;
                    }
                },
            }

            self.area = terminal.get_frame().area();  // ig this is a thing
            self.HandleKeyEvents(&keyParser, &mut clipboard);
            self.HandleMouseEvents(&keyParser);  // not sure if this will be delayed but i think it should work? idk
            keyParser.ClearEvents();
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn HandleMouseEvents (&mut self, events: &KeyParser) {
        if let Some(event) = &events.mouseEvent {
            match event.eventType {
                MouseEventType::Down => {
                    if event.position.0 > 29 && event.position.1 < 10 + self.area.height && event.position.1 > 2 {
                        let currentTime = std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .expect("Time went backwards...")
                            .as_millis();
                        //let acceleration = (1.0 / ((currentTime - self.lastScrolled) as f64 * 0.5 + 0.3) + 1.0) / 3.0;
                        let acceleration = {
                            let v1 = (currentTime - self.lastScrolled) as f64 * -9.0 + 1.5;
                            if v1 > 1.0/4.0 {  v1  }
                            else {  1.0/4.0  }
                        };
                        
                        self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt += acceleration;  // change based on the speed of scrolling to allow fast scrolling
                        self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled =
                            self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt as isize;
                        
                        self.lastScrolled = currentTime;
                    }
                },
                MouseEventType::Up => {
                    if event.position.0 > 29 && event.position.1 < 10 + self.area.height && event.position.1 > 2 {
                        let currentTime = std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .expect("Time went backwards...")
                            .as_millis();
                        //let acceleration = (1.0 / ((currentTime - self.lastScrolled) as f64 * 0.5 + 0.3) + 1.0) / 3.0;
                        let acceleration = {
                            let v1 = (currentTime - self.lastScrolled) as f64 * -9.0 + 1.5;
                            if v1 > 1.0/4.0 {  v1  }
                            else {  1.0/4.0  }
                        };

                        self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt = {
                            let v1 = self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt - acceleration;
                            let v2 = (self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0 + self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt as usize) as f64 * -1.0;
                            if v1 > v2 {  v1  }
                            else {  v2  }
                        };  // change based on the speed of scrolling to allow fast scrolling
                        self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled =
                            self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt as isize;
                        
                        self.lastScrolled = currentTime;
                    }
                },
                MouseEventType::Left => {
                    // checking for code selection
                    if matches!(event.state, MouseState::Release | MouseState::Hold) {
                        if event.position.0 > 29 && event.position.1 < self.area.height - 10 && event.position.1 > 3 {
                            // updating the highlighting position
                            let cursorEnding = self.codeTabs.tabs[self.codeTabs.currentTab].cursor;

                            let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                            let lineSize = 33 + tab.lines.len().to_string().len();  // account for the length of the total lines
                            let linePos = (std::cmp::max(tab.scrolled as isize + tab.mouseScrolled, 0) as usize +
                                event.position.1.saturating_sub(4) as usize,
                                event.position.0.saturating_sub(lineSize as u16) as usize);
                            tab.cursor = (
                                std::cmp::min(
                                    linePos.0,
                                    tab.lines.len() - 1
                                ),
                                linePos.1.saturating_sub( {
                                    if linePos.0 == tab.cursor.0 && linePos.1 > tab.cursor.1 {
                                        1
                                    } else {  0  }
                                } )
                            );
                            tab.cursor.1 = std::cmp::min(
                                tab.cursor.1,
                                tab.lines[tab.cursor.0].len()
                            );
                            tab.mouseScrolled = 0;
                            tab.mouseScrolledFlt = 0.0;
                            self.appState = AppState::Tabs;
                            self.tabState = TabState::Code;

                            if cursorEnding != tab.cursor && !tab.highlighting
                            {
                                if !tab.highlighting {
                                    tab.cursorEnd = cursorEnding;
                                    tab.highlighting = true;
                                }
                            } else if !tab.highlighting {
                                self.codeTabs.tabs[self.codeTabs.currentTab].highlighting = false;
                            }
                        }
                    } else if matches!(event.state, MouseState::Press) {
                        if event.position.0 > 29 && event.position.1 < self.area.height - 10 && event.position.1 > 3 {
                            let currentTime = std::time::SystemTime::now()
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .expect("Time went backwards...")
                                .as_millis();
                            self.codeTabs.tabs[self.codeTabs.currentTab].pauseScroll = currentTime;
                            // updating the highlighting position
                            if events.ContainsMouseModifier(KeyModifiers::Shift)
                            {
                                if !self.codeTabs.tabs[self.codeTabs.currentTab].highlighting {
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursorEnd =
                                        self.codeTabs.tabs[self.codeTabs.currentTab].cursor;
                                    self.codeTabs.tabs[self.codeTabs.currentTab].highlighting = true;
                                }
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].highlighting = false;
                            }

                            let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                            let lineSize = 33 + tab.lines.len().to_string().len();  // account for the length of the total lines
                            let linePos = (std::cmp::max(tab.scrolled as isize + tab.mouseScrolled, 0) as usize +
                                event.position.1.saturating_sub(4) as usize,
                                event.position.0.saturating_sub(lineSize as u16) as usize);
                            tab.cursor = (
                                std::cmp::min(
                                    linePos.0,
                                    tab.lines.len() - 1
                                ),
                                linePos.1.saturating_sub( {
                                    if linePos.0 == tab.cursor.0 && linePos.1 > tab.cursor.1 {
                                        1
                                    } else {  0  }
                                } )
                            );
                            tab.cursor.1 = std::cmp::min(
                                tab.cursor.1,
                                tab.lines[tab.cursor.0].len()
                            );
                            tab.scrolled += tab.mouseScrolledFlt as usize;
                            tab.mouseScrolled = 0;
                            tab.mouseScrolledFlt = 0.0;
                            self.appState = AppState::Tabs;
                            self.tabState = TabState::Code;
                        } else if event.position.0 <= 29 && event.position.1 < self.area.height - 10 && matches!(self.fileBrowser.fileTab, FileTabs::Outline) {
                            // getting the line clicked on and jumping to it if it's in range
                            // account for the line scrolling/shifting... (not as bad as I thought it would be)
                            let scrollTo = self.fileBrowser.outlineCursor.saturating_sub(((self.area.height - 8) / 2) as usize);
                            let line = std::cmp::min(
                                event.position.1.saturating_sub(3) as usize + scrollTo,
                                self.codeTabs.tabs[self.codeTabs.currentTab].linearScopes.len() - 1
                            );
                            self.fileBrowser.outlineCursor = line;
                            let scopes = &mut self.codeTabs.tabs[self.codeTabs.currentTab].linearScopes[
                                line].clone();
                            scopes.reverse();
                            self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0 = 
                                self.codeTabs.tabs[self.codeTabs.currentTab].scopes.GetNode(
                                    scopes
                            ).start;
                            self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled = 0;
                            self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt = 0.0;
                        } else if event.position.0 > 29 && event.position.1 <= 2 {
                            // tallying the size till the correct tab is found
                            let mut sizeCounted = 29usize;
                            for (index, tab) in self.codeTabs.tabFileNames.iter().enumerate() {
                                sizeCounted += 6 + (index + 1).to_string().len() + tab.len();
                                if sizeCounted >= event.position.0 as usize {
                                    self.codeTabs.currentTab = index;
                                    break;
                                }
                            }
                        }
                    }
                },
                MouseEventType::Middle => {},
                MouseEventType::Right => {},
                _ => {},
            }
        }
    }

    fn HandleKeyEvents (&mut self, keyEvents: &KeyParser, clipBoard: &mut Clipboard) {

        match self.appState {
            AppState::CommandPrompt => {
                for chr in &keyEvents.charEvents {
                    self.currentCommand.push(*chr);
                }

                if keyEvents.ContainsKeyCode(KeyCode::Tab) {
                    if keyEvents.ContainsModifier(KeyModifiers::Shift) &&
                        matches!(self.tabState, TabState::Files) {
                        
                        self.fileBrowser.fileTab = match self.fileBrowser.fileTab {
                            FileTabs::Files => FileTabs::Outline,
                            FileTabs::Outline => FileTabs::Files,
                        }
                    } else {
                        self.tabState = match self.tabState {
                            TabState::Code => TabState::Files,
                            TabState::Files => TabState::Tabs,
                            TabState::Tabs => TabState::Code,
                        }
                    }
                }

                match self.tabState {
                    TabState::Code => {
                        if keyEvents.ContainsKeyCode(KeyCode::Return) {
                            self.appState = AppState::Tabs;
                        }
                    },
                    TabState::Files => {
                        if matches!(self.fileBrowser.fileTab, FileTabs::Outline) {
                            if keyEvents.ContainsKeyCode(KeyCode::Return) && self.currentCommand.is_empty() {
                                let mut nodePath = self.codeTabs.tabs[self.codeTabs.currentTab].linearScopes[
                                    self.fileBrowser.outlineCursor].clone();
                                nodePath.reverse();
                                let node = self.codeTabs.tabs[self.codeTabs.currentTab].scopes.GetNode(
                                        &mut nodePath
                                );
                                let start = node.start;
                                self.codeTabs.tabs[self.codeTabs.currentTab].JumpCursor(start, 1);
                            } else if keyEvents.ContainsKeyCode(KeyCode::Up) {
                                self.fileBrowser.MoveCursorUp();
                            } else if keyEvents.ContainsKeyCode(KeyCode::Down) {
                                self.fileBrowser.MoveCursorDown(
                                    &self.codeTabs.tabs[self.codeTabs.currentTab].linearScopes,
                                    &self.codeTabs.tabs[self.codeTabs.currentTab].scopes);
                            }
                        }
                    },
                    TabState::Tabs => {
                        if keyEvents.ContainsKeyCode(KeyCode::Left) {
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                self.codeTabs.MoveTabLeft()
                            } else {
                                self.codeTabs.TabLeft();
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Right) {
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                self.codeTabs.MoveTabRight()
                            } else {
                                self.codeTabs.TabRight();
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Return) {
                            self.appState = AppState::Tabs;
                            self.tabState = TabState::Code;
                        } else if keyEvents.ContainsKeyCode(KeyCode::Delete) {
                            self.codeTabs.tabs.remove(self.codeTabs.currentTab);
                            self.codeTabs.tabFileNames.remove(self.codeTabs.currentTab);
                            self.codeTabs.currentTab = self.codeTabs.currentTab.saturating_sub(1);
                        }
                    },
                }

                if !self.currentCommand.is_empty() {
                    // quiting
                    if keyEvents.ContainsKeyCode(KeyCode::Return) {
                        if self.currentCommand == "q" {
                            self.Exit();
                        }

                        // jumping command
                        if self.currentCommand.starts_with('[') {
                            // jumping up
                            if let Some(numberString) = self.currentCommand.get(1..) {
                                let number = numberString.parse:: <usize>();
                                if number.is_ok() {
                                    let cursor = self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0;
                                    self.codeTabs.tabs[self.codeTabs.currentTab].JumpCursor(
                                        cursor.saturating_sub(number.unwrap()), 1
                                    );
                                }
                            }
                        } else if self.currentCommand.starts_with(']') {
                            // jumping down
                            if let Some(numberString) = self.currentCommand.get(1..) {
                                let number = numberString.parse:: <usize>();
                                if number.is_ok() {
                                    let cursor = self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0;
                                    self.codeTabs.tabs[self.codeTabs.currentTab].JumpCursor(
                                        cursor.saturating_add(number.unwrap()), 1
                                    );
                                }
                            }
                        }

                        self.currentCommand.clear();
                    } else if keyEvents.ContainsKeyCode(KeyCode::Delete) {
                        self.currentCommand.pop();
                    }
                }
            },
            AppState::Tabs => {
                match self.tabState {
                    TabState::Code => {
                        // making sure command + s or other commands are being pressed
                        if !keyEvents.ContainsModifier(KeyModifiers::Command) {
                            for chr in &keyEvents.charEvents {
                                self.codeTabs.tabs[self.codeTabs.currentTab]
                                    .InsertChars(chr.to_string());
                            }
                        }

                        if keyEvents.ContainsKeyCode(KeyCode::Delete) {
                            let mut numDel = 1;
                            let mut offset = 0;

                            if keyEvents.keyModifiers.contains(&KeyModifiers::Option) {
                                if keyEvents.ContainsModifier(KeyModifiers::Shift) {
                                    numDel = self.codeTabs.tabs[self.codeTabs.currentTab].FindTokenPosRight();
                                    offset = numDel;
                                } else {
                                    numDel = self.codeTabs.tabs[self.codeTabs.currentTab].FindTokenPosLeft();
                                }
                            } else if keyEvents.ContainsModifier(KeyModifiers::Command) {
                                if keyEvents.ContainsModifier(KeyModifiers::Shift) {
                                    numDel = self.codeTabs.tabs[self.codeTabs.currentTab].lines[
                                        self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0
                                    ].len() - self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1;
                                    offset = numDel;
                                } else {
                                    numDel = self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1;
                                }
                            } else if keyEvents.ContainsModifier(KeyModifiers::Shift) {
                                offset = numDel;
                            }

                            self.codeTabs.tabs[
                                self.codeTabs.currentTab
                            ].DelChars(numDel, offset);
                        } else if keyEvents.ContainsKeyCode(KeyCode::Left) {
                            let highlight;
                            if keyEvents.ContainsModifier(KeyModifiers::Shift)
                            {
                                highlight = true;
                                if !self.codeTabs.tabs[self.codeTabs.currentTab].highlighting {
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursorEnd =
                                        self.codeTabs.tabs[self.codeTabs.currentTab].cursor;
                                    self.codeTabs.tabs[self.codeTabs.currentTab].highlighting = true;
                                }
                            } else {
                                highlight = false;
                            }
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorLeftToken();
                            } else if keyEvents.ContainsModifier(KeyModifiers::Command) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt = 0.0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled = 0;
                                // checking if it's the true first value or not
                                let mut indentIndex = 0usize;
                                let cursorLine = self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0;
                                for chr in self.codeTabs.tabs[self.codeTabs.currentTab].lines[cursorLine].chars() {                                    
                                    if chr != ' ' {
                                        break;
                                    } indentIndex += 1;
                                }

                                if self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 <= indentIndex {
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 = 0;
                                } else {
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 = indentIndex;
                                }
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorLeft(1, highlight);
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Right) {
                            let highlight;
                            if keyEvents.ContainsModifier(KeyModifiers::Shift)
                            {
                                highlight = true;
                                if !self.codeTabs.tabs[self.codeTabs.currentTab].highlighting {
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursorEnd =
                                        self.codeTabs.tabs[self.codeTabs.currentTab].cursor;
                                    self.codeTabs.tabs[self.codeTabs.currentTab].highlighting = true;
                                }
                            } else {
                                highlight = false;
                            }
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorRightToken();
                            } else if keyEvents.ContainsModifier(KeyModifiers::Command) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt = 0.0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled = 0;

                                let cursorLine = self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 =
                                    self.codeTabs.tabs[self.codeTabs.currentTab].lines[cursorLine].len();
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorRight(1, highlight);
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Up) {
                            let highlight;
                            if keyEvents.ContainsModifier(KeyModifiers::Shift)
                            {
                                highlight = true;
                                if !self.codeTabs.tabs[self.codeTabs.currentTab].highlighting {
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursorEnd =
                                        self.codeTabs.tabs[self.codeTabs.currentTab].cursor;
                                    self.codeTabs.tabs[self.codeTabs.currentTab].highlighting = true;
                                }
                            } else {
                                highlight = false;
                            }
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                                let mut jumps = tab.scopeJumps[tab.cursor.0].clone();
                                jumps.reverse();
                                tab.JumpCursor( 
                                    tab.scopes.GetNode(&mut jumps).start, 1
                                );
                            } else if keyEvents.ContainsModifier(KeyModifiers::Command) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt = 0.0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled = 0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0 = 0;
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].CursorUp(highlight);
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Down) {
                            let highlight;
                            if keyEvents.ContainsModifier(KeyModifiers::Shift)
                            {
                                highlight = true;
                                if !self.codeTabs.tabs[self.codeTabs.currentTab].highlighting {
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursorEnd =
                                        self.codeTabs.tabs[self.codeTabs.currentTab].cursor;
                                    self.codeTabs.tabs[self.codeTabs.currentTab].highlighting = true;
                                }
                            } else {
                                highlight = false;
                            }
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                                let mut jumps = tab.scopeJumps[tab.cursor.0].clone();
                                jumps.reverse();
                                tab.JumpCursor( tab.scopes.GetNode(&mut jumps).end, 1);
                            } else if keyEvents.ContainsModifier(KeyModifiers::Command) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt = 0.0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled = 0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0 = 
                                    self.codeTabs.tabs[self.codeTabs.currentTab].lines.len() - 1;
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].CursorDown(highlight);
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Tab) {
                            if keyEvents.ContainsModifier(KeyModifiers::Shift) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].UnIndent();
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab]
                                    .InsertChars("    ".to_string());
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Return) {
                            self.codeTabs.tabs[self.codeTabs.currentTab].LineBreakIn(false);  // can't be highlighting if breaking?
                        } else if keyEvents.ContainsModifier(KeyModifiers::Command) &&
                            keyEvents.ContainsChar('s') {
                            
                            // saving the program
                            self.codeTabs.tabs[self.codeTabs.currentTab].Save();
                        } else if keyEvents.ContainsModifier(KeyModifiers::Command) &&
                            keyEvents.charEvents.contains(&'c')
                        {
                            // get the highlighted section of text.... or the line if none
                            let text = self.codeTabs.tabs[self.codeTabs.currentTab].GetSelection();
                            let _ = clipBoard.set_text(text);
                        } else if keyEvents.ContainsModifier(KeyModifiers::Command) &&
                        keyEvents.charEvents.contains(&'x')
                        {
                            // get the highlighted section of text.... or the line if none
                            let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                            let text = tab.GetSelection();
                            let _ = clipBoard.set_text(text);

                            // clearing the rest of the selection
                            if tab.highlighting {
                                tab.DelChars(0, 0);
                            } else {
                                tab.lines[tab.cursor.0].clear();
                                tab.RecalcTokens(tab.cursor.0);
                                (tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens);
                            }
                        } else if keyEvents.ContainsModifier(KeyModifiers::Command) &&
                        keyEvents.charEvents.contains(&'v')
                        {
                            // pasting in the text
                            if let Ok(text) = clipBoard.get_text() {
                                let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                                let splitText = text.split('\n');
                                let splitLength = splitText.clone().count() - 1;
                                for (i, line) in splitText.enumerate() {
                                    if line.is_empty() {  continue;  }
                                    tab.InsertChars(
                                        line.to_string()
                                    );
                                    if i < splitLength {
                                        // why does highlight need to be set to true?????? This makes noooo sense??? I give up
                                        tab.LineBreakIn(true);
                                    }
                                }
                            }
                        }
                    },
                    _ => {}  // the other two shouldn't be accessable during the tab state (only during command-line)
                }
            }
        }
        
        // handling escape (switching tabs)
        if keyEvents.ContainsKeyCode(KeyCode::Escape) {
            self.appState = match self.appState {
                AppState::Tabs => {
                    self.tabState = TabState::Files;

                    AppState::CommandPrompt
                },
                AppState::CommandPrompt => {
                    if matches!(self.tabState, TabState::Files | TabState::Tabs) {
                        self.tabState = TabState::Code;
                    }

                    AppState::Tabs
                },
            }
        }
    }

    fn Exit(&mut self) {
        self.exit = true;
    }

}


impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        // ============================================= file block here =============================================
        let mut tabBlock = Block::bordered()
            .border_set(border::THICK);
        if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Tabs) {
            tabBlock = tabBlock.light_blue();
        }

        let coloredTabText = self.codeTabs.GetColoredNames(
            matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Tabs)
        );
        let tabText = Text::from(vec![
            Line::from(coloredTabText)
        ]);

        Paragraph::new(tabText)
            .block(tabBlock)
            .render(Rect {
                x: area.x + 29,
                y: area.y,
                width: area.width - 20,
                height: 3
        }, buf);


        // ============================================= code block here =============================================
        let codeBlockTitle = Line::from(vec![
            " ".to_string().white(),
            self.codeTabs.tabs[
                self.codeTabs.currentTab
            ].name
            .clone()
            .bold(),
            " ".to_string().white(),
        ]);
        let mut codeBlock = Block::bordered()
            .title_top(codeBlockTitle.centered())
            .border_set(border::THICK);
        if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Code) {
            codeBlock = codeBlock.light_blue();
        }

        let codeText = Text::from(
            self.codeTabs.GetScrolledText(
                area,
                matches!(self.appState, AppState::Tabs) &&
                    matches!(self.tabState, TabState::Code)
            )
        );

        Paragraph::new(codeText)
            .block(codeBlock)
            .render(Rect {
                x: area.x + 29,
                y: area.y + 2,
                width: area.width - 29,
                height: area.height - 10
        }, buf);


        // ============================================= files =============================================
        let mut fileBlock = Block::bordered()
            .border_set(border::THICK);
        if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Files) {
            fileBlock = fileBlock.light_blue();
        }

        let mut fileStringText = vec!();
        let mut scopes: Vec <usize> = vec![];

        let fileText: Text;

        if matches!(self.fileBrowser.fileTab, FileTabs::Outline) {
            let mut newScroll = self.fileBrowser.outlineCursor;
            let mut scrolled = 0;
            let scrollTo = self.fileBrowser.outlineCursor.saturating_sub(((area.height - 8) / 2) as usize);
            for scopeIndex in &self.codeTabs.tabs[self.codeTabs.currentTab].scopeJumps {
                if {
                    let mut valid = true;
                    for i in 0..scopes.len() {
                        let slice = scopes.get(0..(scopes.len()-i));
                        if slice.unwrap_or(&[]) == *scopeIndex {
                            valid = false;
                            break;
                        }
                    }
                    valid
                } {
                    scopes.clear();

                    let mut scope = &self.codeTabs.tabs[self.codeTabs.currentTab].scopes;
                    for index in scopeIndex {
                        scopes.push(*index);
                        scope = &scope.children[*index];
                    }
                    if scopeIndex.is_empty() {  continue;  }
                    scrolled += 1;
                    if *scopeIndex ==
                    self.codeTabs.tabs[self.codeTabs.currentTab]
                        .scopeJumps[self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0] &&
                        matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) {
                
                        newScroll = scrolled - 1;
                    }
                    if scrolled < scrollTo {  continue;  }
                    fileStringText.push(
                        Line::from(vec![
                            {
                                let mut offset = String::new();
                                if *scopeIndex ==
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .scopeJumps[self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0] &&
                                        matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) {
                                
                                    offset.push('>')
                                } else if
                                    matches!(self.appState, AppState::CommandPrompt) &&
                                    matches!(self.tabState, TabState::Files) &&
                                    self.codeTabs.tabs[self.codeTabs.currentTab].linearScopes[
                                        self.fileBrowser.outlineCursor
                                    ] == *scopeIndex {
                                    offset.push('>');
                                }
                                for _ in 0..scopeIndex.len().saturating_sub(1) {
                                    offset.push_str("  ");
                                }
                                
                                offset.white()
                            },
                            {
                                if *scopeIndex ==
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .scopeJumps[self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0] &&
                                    matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Code) {
                                    
                                    match scopeIndex.len() {
                                        1 => scope.name.clone().light_blue(),
                                        2 => scope.name.clone().light_magenta(),
                                        3 => scope.name.clone().light_red(),
                                        4 => scope.name.clone().light_yellow(),
                                        5 => scope.name.clone().light_green(),
                                        _ => scope.name.clone().white(),
                                    }.underlined()
                                } else if
                                    matches!(self.appState, AppState::CommandPrompt) &&
                                    matches!(self.tabState, TabState::Files) &&
                                    self.codeTabs.tabs[self.codeTabs.currentTab].linearScopes[
                                        self.fileBrowser.outlineCursor
                                    ] == *scopeIndex {
                                    
                                    match scopeIndex.len() {
                                        1 => scope.name.clone().light_blue().underlined(),
                                        2 => scope.name.clone().light_magenta().underlined(),
                                        3 => scope.name.clone().light_red().underlined(),
                                        4 => scope.name.clone().light_yellow().underlined(),
                                        5 => scope.name.clone().light_green().underlined(),
                                        _ => scope.name.clone().white().underlined(),
                                    }
                                } else {
                                    match scopeIndex.len() {
                                        1 => scope.name.clone().light_blue(),
                                        2 => scope.name.clone().light_magenta(),
                                        3 => scope.name.clone().light_red(),
                                        4 => scope.name.clone().light_yellow(),
                                        5 => scope.name.clone().light_green(),
                                        _ => scope.name.clone().white(),
                                    }
                                }
                            },
                            //format!(" ({}, {})", scope.start + 1, scope.end + 1).white(),  // (not enough space for it to fit...)
                        ])
                    );
                }
            }
            self.fileBrowser.outlineCursor = newScroll;
            fileText = Text::from(fileStringText);
        } else {
            let mut allFiles = vec!();
            for (index, file) in self.fileBrowser.files.iter().enumerate() {
                allFiles.push(Line::from(vec![
                    {
                        if index == self.fileBrowser.fileCursor {
                            file.clone().white().underlined()
                        } else {
                            file.clone().white()
                        }
                    }
                ]));
            }
            fileText = Text::from(allFiles);
        }

        Paragraph::new(fileText)
            .block(fileBlock)
            .render(Rect {
                x: area.x,
                y: area.y,
                width: 30,
                height: area.height - 8
        }, buf);

            
        // ============================================= Error Bar =============================================
        let errorBlock = Block::bordered()
            .border_set(border::THICK);
        
        let errorText = Text::from(vec![
            Line::from(vec![
                format!("Debug: {}", self.debugInfo).red().bold(),
                //"Error: callback on line 5".to_string().red().bold()
            ]),
        ]);

        Paragraph::new(errorText)
            .block(errorBlock)
            .render(Rect {
                x: area.x,
                y: area.y + area.height - 9,
                width: area.width,
                height: 8
            }, buf);
        
        // ============================================= Commandline =============================================
        let commandText = Text::from(vec![
            Line::from(vec![
                "/".to_string().white().bold(),
                self.currentCommand.clone().white().italic(),
                {
                    if matches!(self.appState, AppState::CommandPrompt) {
                        "_".to_string().white().slow_blink().bold()
                    } else {
                        "".to_string().white()
                    }
                }
            ])
        ]);

        Paragraph::new(commandText)
            .render(Rect {
                x: area.x + 2,
                y: area.y + area.height - 1,
                width: area.width,
                height: 1
            }, buf);
    }
}


/*
Commands: √ <esc>
    √ <enter> -> exit commands
    √ <q> + <enter> -> exit application
    √ <tab> -> switch tabs:

        * code editor:
            <cmd> + <1 - 9> -> switch to corresponding code tab
            
            √ <left/right/up/down> -> movement in the open file
            √ <option> + <left/right> -> jump to next token
            √ <option> + <up/down> -> jump to end/start of current scope
            √ <cmnd> + <left/right> -> jump to start/end of line
                √ - first left jumps to the indented start
                √ - second left jumps to the true start
            √ <cmnd> + <up/down> -> jump to start/end of file
            √ <shift> + any line movement/jump -> highlight all text selected (including jumps from line # inputs)
            <ctrl> + <[> -> temporarily opens command line to input number of lines to jump up
            <ctrl> + <]> -> temporarily opens command line to input number of lines to jump down
            <ctrl> + <1-9> -> jump up that many positions
            <ctrl> + <1-9> + <shift> -> jump down that many positions

            <ctrl> + <left>/<right> -> open/close scope of code (can be done from any section inside the scope, not just the start/end)

            <cmnd> + <c> -> copy (either selection or whole line when none)
            <cmnd> + <v> -> paste to current level (align relative indentation to the cursor's)

            √ <del> -> deletes the character the cursor is on
            √ <del> + <option> -> delete the token the cursor is on
            √ <del> + <cmnd> -> delete the entire line
            √ <del> + <shift> + <cmnd/option/none> -> does the same as specified before execpt to the right instead

            <tab> -> indents the line up to the predicted indentation
            √ <tab> + <shift> -> unindents the line by one
            <enter> -> creates a new line
            <enter> + <cmnd> -> creates a new line starting at the end of the current
            <enter> + <cmnd> + <shift> -> creates a new line starting before the current

        * file browser / program outline:
            <up/down> -> move between files/folders
            <enter> -> open in new tab
            <right> -> open settings menu:
                <up/down> -> move between settings
                <enter> -> edit setting
                <left> -> close menu
            
            √ <shift> + <tab> -> cycle between pg outline and file broswer

            outline:
                √ - shows all functions/methods/classes/etc... so they can easily be acsess without needed the mouse and without wasting time scrolling

                √ <enter> -> jumps the cursor to that section in the code
                <shift> + <enter> -> jumps the cursor to the section and switches to code editing
                
                √ <down/up> -> moves down or up one
                <option> + <down/up> -> moves up or down to the start/end of the current scope
                <cmnd> + <down>/<up> -> moves to the top or bottom of the outline
                
                <ctrl> + <left> -> collapse the scope currently selected
                <ctrl> + <right> -> uncollapse the scope

        * code tabs:
            √ <left/right> -> change tab
            √ <option> + <left/right> -> move current tab left/right
            √ <del> -> close current tab

the bottom bar is 4 lines tall (maybe this could be a custom parameter?)
the side bar appears behind the bottom bar and pops outward shifting the text

Errors underlined in red
Warnings underlined in yellow
    integrate a way to run clippy on the typed code
    parse clippy's output into the proper warning or errors
    display the error or warning at the bottom (where code completion suggestions go)

Suggestions appear on the very bottom as to not obstruct the code being written
*/

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    enableMouseCapture().await;
    let app_result = App::default().run(&mut terminal).await;
    disableMouseCapture().await;
    ratatui::restore();
    app_result
}

