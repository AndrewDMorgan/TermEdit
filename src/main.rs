// snake case is just bad
#![allow(non_snake_case)]

use tokio::io::{self, AsyncReadExt};
use vte::Parser;

use crossterm::terminal::enable_raw_mode;
use arboard::Clipboard;

mod CodeTabs;
mod Tokens;
mod eventHandler;

use eventHandler::*;
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
use eventHandler::{KeyCode, KeyModifiers, KeyParser, MouseEventType};

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
                    let mut lineNumber = 0;
                    let ending = tab.fileName.split('.').last().unwrap_or("");
                    for line in tab.lines.iter() {
                        tab.lineTokenFlags.push(vec!());
                        tab.lineTokens.push(
                            {
                                GenerateTokens(line.clone(),
                                               ending,
                                               &mut tab.lineTokenFlags,
                                               lineNumber,
                                               &mut tab.outlineKeywords
                                )
                            }
                        );
                        lineNumber += 1;
                    }
                    (tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);

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
    suggested: String
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
        let mut buffer = [0; 128];  // [0; 10]; not sure how much the larger buffer is actually helping
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
                _ = tokio::time::sleep(std::time::Duration::from_millis({
                    let currentTime = std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .expect("Time went backwards...")
                            .as_millis();
                    if currentTime - self.lastScrolled < 200 {10}
                    else if currentTime - keyParser.lastPress > 750 {50}  // hopefully this will help with cpu usage
                    else {5}  // this bit of code is a mess...
                })) => {
                    terminal.draw(|frame| self.draw(frame))?;
                    if self.exit {
                        break;
                    }
                },
            }
            
            self.area = terminal.get_frame().area();  // ig this is a thing
            self.HandleKeyEvents(&keyParser, &mut clipboard);
            self.HandleMouseEvents(&keyParser);  // not sure if this will be delayed, but I think it should work? idk
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
                        if event.position.0 > 29 && event.position.1 < self.area.height - 8 && event.position.1 > 3 {
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
                        if event.position.0 > 29 && event.position.1 < self.area.height - 8 && event.position.1 > 3 {
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
                            tab.scrolled = std::cmp::max(tab.mouseScrolledFlt as isize + tab.scrolled as isize, 0) as usize;
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
                                if *chr == '(' {
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .InsertChars("()".to_string());
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 -= 1;
                                } else if *chr == '{' {
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .InsertChars("{}".to_string());
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 -= 1;
                                } else if *chr == '[' {
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .InsertChars("[]".to_string());
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 -= 1;
                                } else if *chr == '\"' {
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .InsertChars("\"\"".to_string());
                                    self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 -= 1;
                                } else {
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .InsertChars(chr.to_string());
                                }
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
                                let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                                tab.scrolled = std::cmp::max(tab.mouseScrolledFlt as isize + tab.scrolled as isize, 0) as usize;
                                tab.mouseScrolledFlt = 0.0;
                                tab.mouseScrolled = 0;

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
                                let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                                tab.scrolled = std::cmp::max(tab.mouseScrolledFlt as isize + tab.scrolled as isize, 0) as usize;
                                tab.mouseScrolledFlt = 0.0;
                                tab.mouseScrolled = 0;
                                tab.cursor.0 = 0;
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
                                let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                                tab.scrolled = std::cmp::max(tab.mouseScrolledFlt as isize + tab.scrolled as isize, 0) as usize;
                                tab.mouseScrolledFlt = 0.0;
                                tab.mouseScrolled = 0;
                                tab.cursor.0 = 
                                    tab.lines.len() - 1;
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].CursorDown(highlight);
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Tab) {
                            if keyEvents.ContainsModifier(KeyModifiers::Shift) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].UnIndent();
                            } else {
                                if self.suggested.is_empty() {
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .InsertChars("    ".to_string());
                                } else {
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .RemoveCurrentToken_NonUpdate();
                                    self.codeTabs.tabs[self.codeTabs.currentTab]
                                        .InsertChars(self.suggested.clone());
                                }
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
                                (tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);
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
                                    if i > 0 {
                                        // making sure all actions occur on the same iteration
                                        if let Some(mut elements) = tab.changeBuffer.pop() {
                                            while let Some(element) = elements.pop() {
                                                let size = tab.changeBuffer.len() - 1;
                                                tab.changeBuffer[size].insert(0, element);
                                            }
                                        }
                                    }
                                    if i < splitLength {
                                        // why does highlight need to be set to true?????? This makes noooo sense??? I give up
                                        tab.LineBreakIn(true);
                                        // making sure all actions occur on the same iteration
                                        if let Some(mut elements) = tab.changeBuffer.pop() {
                                            while let Some(element) = elements.pop() {
                                                let size = tab.changeBuffer.len() - 1;
                                                tab.changeBuffer[size].insert(0, element);
                                            }
                                        }
                                    }
                                }
                            }
                        } else if keyEvents.ContainsModifier(KeyModifiers::Command) &&
                            keyEvents.charEvents.contains(&'f')
                        {
                            // finding the nearest occurrence to the cursor
                            let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                            if tab.highlighting {
                                // getting the new search term (yk, it's kinda easy when done the right way the first time......not happening again though)
                                let selection = tab.GetSelection();
                                tab.searchTerm = selection;
                            }

                            // searching for the term
                            let mut lastDst = (usize::MAX, 0usize);
                            for (index, line) in tab.lines.iter().enumerate() {
                                let dst = (index as isize - tab.cursor.0 as isize).saturating_abs() as usize;
                                if !line.contains(&tab.searchTerm) {  continue;  }
                                if dst > lastDst.0 {  break;  }
                                lastDst = (dst, index);
                            }

                            if lastDst.0 < usize::MAX {
                                tab.searchIndex = lastDst.1;
                                //self.debugInfo = lastDst.1.to_string();
                            }
                        } else if keyEvents.ContainsModifier(KeyModifiers::Command) &&
                            keyEvents.charEvents.contains(&'z')
                        {
                            if keyEvents.ContainsModifier(KeyModifiers::Shift) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].Redo();
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].Undo();
                            }
                        }
                    },
                    _ => {}  // the other two shouldn't be accessible during the tab state (only during command-line)
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

        // temp todo! replace elsewhere (the sudo auto-checker is kinda crap tbh)
        self.debugInfo.clear();
        /*
        for var in &self.codeTabs.tabs[self.codeTabs.currentTab].outlineKeywords {
            if matches!(var.kwType, OutlineType::Function) {
                self.debugInfo.push('(');
                self.debugInfo.push_str(var.keyword.as_str());
                self.debugInfo.push('/');
                self.debugInfo.push_str(&format!("{:?}", var.scope));
                self.debugInfo.push(')');
            }
        }*/
        self.suggested.clear();
        let mut scope = self.codeTabs.tabs[self.codeTabs.currentTab].scopeJumps[
            self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0
            ].clone();
        let mut tokenSet: Vec <String> = vec!();
        self.codeTabs.tabs[self.codeTabs.currentTab].GetCurrentToken(&mut tokenSet);
        if !tokenSet.is_empty() {  // token set is correct it seems
            let token = tokenSet.remove(0);  // getting the item actively on the cursor
            // self.debugInfo.push_str("{");
            // self.debugInfo.push_str(&token);
            // self.debugInfo.push_str("}");
            let mut currentScope =
                self.codeTabs.tabs[self.codeTabs.currentTab].scopeJumps[
                self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0
            ].clone();
            if !tokenSet.is_empty() {
                let mut currentElement = OutlineKeyword::TryFindKeyword(
                    &self.codeTabs.tabs[self.codeTabs.currentTab].outlineKeywords,
                    tokenSet.pop().unwrap()
                );
                if let Some(set) = &currentElement {
                    let newScope = self.codeTabs.tabs[self.codeTabs.currentTab].scopeJumps[
                        set.lineNumber
                    ].clone();
                    //self.debugInfo.push_str(&format!("{:?} ", newScope.clone()));
                    currentScope = newScope;
                }

                while !tokenSet.is_empty() && currentElement.is_some() {
                    //self.debugInfo.push(' ');
                    let newToken = tokenSet.remove(0);
                    if let Some(set) = currentElement {
                        let newScope = self.codeTabs.tabs[self.codeTabs.currentTab].scopeJumps[
                            set.lineNumber
                        ].clone();
                        //self.debugInfo.push_str(&format!("{:?} ", newScope.clone()));
                        currentScope = newScope;
                        currentElement = OutlineKeyword::TryFindKeyword(&set.childKeywords, newToken);
                    }
                }
            }
            scope = currentScope.clone();
            if !matches!(token.as_str(), " " | "," | "|" | "}" | "{" | "[" | "]" | "(" | ")" |
                        "+" | "=" | "-" | "_" | "!" | "?" | "/" | "<" | ">" | "*" | "&" |
                        ".")
            {
                let validKeywords = OutlineKeyword::GetValidScoped(
                    &self.codeTabs.tabs[self.codeTabs.currentTab].outlineKeywords,
                    &scope
                );

                let mut closest = (usize::MAX, "".to_string(), "".to_string());
                for var in validKeywords {
                    //*
                    if matches!(var.kwType, OutlineType::Function) {
                        self.debugInfo.push('(');
                        self.debugInfo.push_str(var.keyword.as_str());
                        self.debugInfo.push('/');
                        self.debugInfo.push_str(&format!("{:?}", var.scope));
                        self.debugInfo.push(')');
                    }  // */
                    let value = WordComparison(&token, &var.keyword);
                    if value < closest.0 {
                        let mut t = String::new();
                        if matches!(var.kwType, OutlineType::Function | OutlineType::Enum) && false {  // basic printing of parameters
                            //t.push('(');
                            //t.push_str(var.keyword.as_str());
                            //t.push('/');
                            //t.push_str(&format!(":{:?}", var.parameters));
                            /*
                            for child in var.childKeywords {
                                t.push_str(child.keyword.as_str());
                                t.push(',');
                            } // */
                            //t.push(')');
                        }
                        closest = (value, var.keyword.clone(), t);
                    }
                }
                if closest.0 < 15 && closest.1 != token.as_str() {  // todo! keep all of this up to date
                    self.suggested = closest.1;
                    //self.suggested.push_str(closest.0.to_string().as_str());
                    //self.debugInfo.push(' ');
                    //self.debugInfo.push_str(closest.0.to_string().as_str());
                    //self.debugInfo.push_str(" / ");
                    self.debugInfo.push_str(closest.2.as_str());
                }
            }
        }

        let errorText = Text::from(vec![
            Line::from(vec![
                format!(": {}?", self.suggested).white().italic(),
            ]),
            Line::from(vec![
                format!("Debug: {}", self.debugInfo).red().bold(),
                format!(" ; {:?}", scope).white()
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

// kinda bad but kinda sometimes works; at least it should be fairly quick
fn WordComparison (wordMain: &String, wordComp: &String) -> usize {
    let mut totalError = 0;
    let wordBytes = wordComp.as_bytes();
    for (index, byte) in wordMain
        .bytes()
        .enumerate()
    {
        if index >= wordComp.len() {  break;  }
        totalError += (byte as i8 - wordBytes[index] as i8)
            .abs() as usize;
    }

    if wordComp.len() < wordMain.len() {
        totalError += (wordMain.len() - wordComp.len()) * 2;
    }

    totalError
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

            √ <cmnd> + <c> -> copy (either selection or whole line when none)
            √ <cmnd> + <v> -> paste to current level (align relative indentation to the cursor's)

            √ <del> -> deletes the character the cursor is on
            √ <del> + <option> -> delete the token the cursor is on
            √ <del> + <cmnd> -> delete the entire line
            √ <del> + <shift> + <cmnd/option/none> -> does the same as specified before except to the right instead

            <tab> -> indents the lineup to the predicted indentation
            √ <tab> + <shift> -> unindents the line by one
            √<enter> -> creates a new line
            <enter> + <cmnd> -> creates a new line starting at the end of the current
            <enter> + <cmnd> + <shift> -> creates a new line starting before the current

        * file browser / program outline:
            <up/down> -> move between files/folders
            <enter> -> open in new tab
            <right> -> open settings menu:
                <up/down> -> move between settings
                <enter> -> edit setting
                <left> -> close menu
            
            √ <shift> + <tab> -> cycle between pg outline and file browser

            outline:
                √ - shows all functions/methods/classes/etc... so they can easily be access without needed the mouse and without wasting time scrolling

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
the sidebar appears behind the bottom bar and pops outward shifting the text

Errors underlined in red
Warnings underlined in yellow
    integrate a way to run clippy on the typed code
    parse Clippy's output into the proper warning or errors
    display the error or warning at the bottom (where code completion suggestions go)

Suggestions appear on the very bottom as to not obstruct the code being written



add undo/redo
maybe show the outline moving while scrolling?
Add scrolling to the outline
make it so when indenting/unindenting it does it for the entire selection if highlighting
make it so when pressing return, it auto sets to the correct indent level
    same thing for various other actions being set to the correct indent level

Fix the highlighting bug when selecting text above and only partially covering a token; It's not rendering the full token and does weird things...
    It has something to do with the end and starting cursor char positions being aligned on the seperate lines
Add double-clicking to highlight a token or line
make it so that highlighting than typing () or {} or [] encloses the text rather than replacing it

command f
    Add the ability to type in custom search terms through the command line
    Add the ability to scroll through the various options
    Currently, detection seems to be working at least for single line terms
    Make it jump the cursor to the terms

settings menu for setting alternative keybindings? or how should that be done?
general settings menu
directory selection

make command + x properly shift the text onto a new line when pasted (along w/ being at the correct indent level)

make it so that the outline menu, when opened, is placed at the correct location rather than defaulting to the start until re-entering the code tab

maybe move all the checks for the command key modifier to a single check that then corresponds to other checks beneath it? I'm too lazy rn though

Prevent the program from crashing when touch non-u8 characters (either handle the error, or do something but don't crash)
    Ideally the user can still copy and past them along with placing them and deleting them

Add syntax highlighting for python

Make the undo/redo undo/redo copy/paste in one move instead of multiple

Fix multi-line comments:
    maybe set each line to a state such as: Null, Comment Next
    this could be read and edited based on the current line (only some updates would have to propagate)

Fix the bug with the scope outline system where it clicks one cell too high when it hasn't been scrolled yet

Fix the bug with highlighting; when highlighting left and pressing right arrow, it stays at the left
    Highlighting right works fine though, idk why

Allow language highlighting to determine unique characters for comments and multi line comments


option + tab = accept auto complete suggestion

either check the token the mouse is on or to the left (if partially on one consider the whole token)
based on that token, find the closest possibility assuming it falls within a certain error range
account for members vs. methods vs. functions vs. variables

make a better system that can store keybindings; the user can make a custom one, or there are two defaults: mac custom, standard

(complete) make any edits cascade down the file (such as multi line comments) and update those lines until terminated
    (incomplete..... me no want do) Figure out a way to determine if the set is complete so it doesn't  update the whole file
    Todo! Error! Fix the variable checker/saver to handle when variables are removed from the code, idk how; have fun :(

*/


#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    eventHandler::enableMouseCapture().await;
    let app_result = App::default().run(&mut terminal).await;
    eventHandler::disableMouseCapture().await;
    ratatui::restore();
    app_result
}


