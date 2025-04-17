// snake case is just bad
#![allow(non_snake_case)]

use std::path::PathBuf;
use tokio::io::{self, AsyncReadExt};
use vte::Parser;

use crossterm::terminal::enable_raw_mode;
use arboard::Clipboard;  // for copy + paste + cut
use dirs::home_dir;  // auto gets the starting file path

mod CodeTabs;
mod Tokens;
mod eventHandler;
mod StringPatternMatching;
mod Colors;

use StringPatternMatching::*;
use eventHandler::*;
use Colors::Colors::*;
use Tokens::*;

use CodeTabs::CodeTab;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use ratatui::prelude::Alignment;
use eventHandler::{KeyCode, KeyModifiers, KeyParser, MouseEventType};

#[derive(Debug, Default)]
pub enum FileTabs {
    Outline,
    #[default] Files,
}

#[derive(Debug, Default)]
pub struct FileBrowser {
    files: Vec <String>,  // stores the names
    filePaths: Vec <String>,

    fileTab: FileTabs,
    fileCursor: usize,
    outlineCursor: usize,
}

static VALID_EXTENSIONS: [&str; 9] = [
    "txt",
    "rs",
    "py",
    "cpp",
    "hpp",
    "c",
    "h",
    "lua",
    "toml",
];

// manages the files for a given project
// provides an outline and means for loading files
impl FileBrowser {
    // gets the complete path name
    pub fn GetPathName (dirSuffix: &str) -> String {
        home_dir()
            .unwrap_or(PathBuf::from("/"))
            .join(dirSuffix)
            .to_string_lossy()
            .into_owned()
    }

    // finds all directories in a given directory
    pub fn CalculateDirectories (directory: &String, nextDirectories: &mut Vec <String>) {
        if let Ok(paths) = std::fs::read_dir(directory) {
            for path in paths.flatten() {
                if std::fs::FileType::is_dir(&path.file_type().unwrap()) {
                    nextDirectories.push(path
                        .file_name()
                        .to_str()
                        .unwrap_or("")
                        .to_string()
                    );
                }
            }
        }
    }

    // loads a project into memory
    pub fn LoadFilePath (
        &mut self,
        indirectPathInput: &str,
        codeTabs: &mut CodeTabs::CodeTabs
    ) -> io::Result <()> {
        self.files.clear();
        codeTabs.tabs.clear();
        let pathInput = home_dir()
            .unwrap_or(PathBuf::from("/"))
            .join(indirectPathInput)
            .to_string_lossy()
            .into_owned();
        if let Ok(paths) = std::fs::read_dir(pathInput.clone()) {
            for path in paths.flatten() {
                if std::fs::FileType::is_file(&path.file_type().unwrap()) {
                    let name = path.file_name().to_str().unwrap_or("").to_string();

                    // so it doesn't try and load invalid files
                    if !VALID_EXTENSIONS.contains(&name.split(".").last().unwrap_or("")) {  continue;  }

                    self.files.push(name.clone());
                    let mut fullPath = pathInput.clone();
                    fullPath.push_str(&name);
                    self.filePaths.push(fullPath);
                }
            } Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Failed to find directory"))
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
    Tabs,
    CommandPrompt,
    #[default] Menu,
}

#[derive(Debug, Default)]
pub enum MenuState {
    #[default] Welcome,
    Settings
}


#[derive(Debug, Default)]
pub struct App <'a> {
    exit: bool,
    appState: AppState,
    tabState: TabState,
    codeTabs: CodeTabs::CodeTabs,
    currentCommand: String,
    fileBrowser: FileBrowser,
    area: Rect,
    lastScrolled: u128,

    debugInfo: String,
    suggested: String,

    preferredCommandKeybind: KeyModifiers,
    colorMode: ColorMode <'a>,

    menuState: MenuState,

    currentDir: String,
    dirFiles: Vec<String>,

    currentMenuSettingBox: usize,

    lastTab: usize,

    luaSyntaxHighlightScripts: std::collections::HashMap <Languages, mlua::Function>,
}

impl <'a> App <'a> {

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        enable_raw_mode()?; // Enable raw mode for direct input handling

        // loading the lua syntax highlighting scripts
        // make this a procedural macro to avoid repetitive writing
        let lua = mlua::Lua::new();
        lua.load(
            std::fs::read_to_string("assets/nullSyntaxHighlighting.lua")?
        ).exec().unwrap();

        self.luaSyntaxHighlightScripts.insert(
            Languages::Null,
                lua.globals().get("GetTokens").unwrap()
        );

        // rust
        let lua = mlua::Lua::new();
        lua.load(
            std::fs::read_to_string("assets/rustSyntaxHighlighting.lua")?
        ).exec().unwrap();

        self.luaSyntaxHighlightScripts.insert(
            Languages::Null,
                lua.globals().get("GetTokens").unwrap()
        );

        // lua
        let lua = mlua::Lua::new();
        lua.load(
            std::fs::read_to_string("assets/luaSyntaxHighlighting.lua")?
        ).exec().unwrap();

        self.luaSyntaxHighlightScripts.insert(
            Languages::Lua,
            lua.globals().get("GetTokens").unwrap()
        );

        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        
        let mut clipboard = Clipboard::new().unwrap();

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
            self.HandleKeyEvents(&keyParser, &mut clipboard).await;
            self.HandleMouseEvents(&keyParser).await;  // not sure if this will be delayed, but I think it should work? idk
            keyParser.ClearEvents();
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn HandleUpScroll (&mut self, event: &MouseEvent) {
        if event.position.0 > 29 && event.position.1 < 10 + self.area.height && event.position.1 > 2 {
            let mut tabIndex = 0;
            tabIndex = self.codeTabs.GetTabNumber(
                &self.area, 29,
                event.position.0 as usize,
                &mut tabIndex
            );

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

            self.codeTabs.tabs[tabIndex].mouseScrolledFlt = {
                let v1 = self.codeTabs.tabs[tabIndex].mouseScrolledFlt - acceleration;
                let v2 = (self.codeTabs.tabs[tabIndex].cursor.0 + self.codeTabs.tabs[tabIndex].mouseScrolledFlt as usize) as f64 * -1.0;
                if v1 > v2 {  v1  }
                else {  v2  }
            };  // change based on the speed of scrolling to allow fast scrolling
            self.codeTabs.tabs[tabIndex].mouseScrolled =
                self.codeTabs.tabs[tabIndex].mouseScrolledFlt as isize;

            self.lastScrolled = currentTime;
        }
    }

    fn HandleDownScroll (&mut self, event: &MouseEvent) {
        if event.position.0 > 29 && event.position.1 < 10 + self.area.height && event.position.1 > 2 {
            let mut tabIndex = 0;
            tabIndex = self.codeTabs.GetTabNumber(
                &self.area, 29,
                event.position.0 as usize,
                &mut tabIndex
            );

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

            self.codeTabs.tabs[tabIndex].mouseScrolledFlt += acceleration;  // change based on the speed of scrolling to allow fast scrolling
            self.codeTabs.tabs[tabIndex].mouseScrolled =
                self.codeTabs.tabs[tabIndex].mouseScrolledFlt as isize;

            self.lastScrolled = currentTime;
        }
    }

    fn PressedCode (&mut self, events: &KeyParser, event: &MouseEvent) {
        self.lastTab = self.codeTabs.GetTabNumber(
            &self.area, 29,
            event.position.0 as usize,
            &mut self.lastTab
        );
        let currentTime = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Time went backwards...")
            .as_millis();
        self.codeTabs.tabs[self.lastTab].pauseScroll = currentTime;
        // updating the highlighting position
        if events.ContainsMouseModifier(KeyModifiers::Shift)
        {
            if !self.codeTabs.tabs[self.lastTab].highlighting {
                self.codeTabs.tabs[self.lastTab].cursorEnd =
                    self.codeTabs.tabs[self.lastTab].cursor;
                self.codeTabs.tabs[self.lastTab].highlighting = true;
            }
        } else {
            self.codeTabs.tabs[self.lastTab].highlighting = false;
        }

        // adjusting the position for panes
        let position = (
            self.codeTabs.GetRelativeTabPosition(event.position.0, self.area, 33),
            event.position.1
        );

        let tab = &mut self.codeTabs.tabs[self.lastTab];
        let lineSize = tab.lines.len().to_string().len();  // account for the length of the total lines
        let linePos = (std::cmp::max(tab.scrolled as isize + tab.mouseScrolled, 0) as usize +
                           position.1.saturating_sub(4) as usize,
                       position.0.saturating_sub(lineSize as u16) as usize);
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
    }

    fn PressedScopeJump (&mut self, _events: &KeyParser, event: &MouseEvent) {
        // getting the line clicked on and jumping to it if it's in range
        // account for the line scrolling/shifting... (not as bad as I thought it would be)
        let scrollTo = self.fileBrowser.outlineCursor.saturating_sub(((self.area.height - 8) / 2) as usize);
        let line = std::cmp::min(
            event.position.1.saturating_sub(3) as usize + scrollTo,
            self.codeTabs.tabs[self.lastTab].linearScopes.len() - 1
        );
        self.fileBrowser.outlineCursor = line;
        let scopes = &mut self.codeTabs.tabs[self.lastTab].linearScopes[
            line].clone();
        scopes.reverse();
        self.codeTabs.tabs[self.lastTab].cursor.0 =
            self.codeTabs.tabs[self.lastTab].scopes.GetNode(
                scopes
            ).start;
        self.codeTabs.tabs[self.lastTab].mouseScrolled = 0;
        self.codeTabs.tabs[self.lastTab].mouseScrolledFlt = 0.0;
    }

    fn PressedNewCodeTab (&mut self, events: &KeyParser, event: &MouseEvent) {
        // tallying the size till the correct tab is found
        let mut sizeCounted = 29usize;
        for (index, tab) in self.codeTabs.tabFileNames.iter().enumerate() {
            sizeCounted += 6 + (index + 1).to_string().len() + tab.len();
            if sizeCounted >= event.position.0 as usize {
                if events.ContainsMouseModifier(KeyModifiers::Shift) {
                    self.codeTabs.panes.push(index);
                } else {
                    self.codeTabs.currentTab = index;
                    self.lastTab = index;
                }
                break;
            }
        }
    }

    async fn PressedLoadFile (&mut self, _events: &KeyParser, event: &MouseEvent) {
        let height = event.position.1.saturating_sub(2) as usize;
        if self.fileBrowser.files.len() > height {
            // loading the file's contents
            /*if !self.codeTabs.tabs.is_empty() {
                self.codeTabs.panes.push(self.codeTabs.tabs.len());
            } else {
                self.codeTabs.currentTab = self.codeTabs.tabs.len();
            }*/
            self.codeTabs.currentTab = self.codeTabs.tabs.len();

            let name = &self.fileBrowser.files[height];

            let mut lines: Vec <String> = vec!();

            let fullPath = &self.fileBrowser.filePaths[height];

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
            tab.name = name.clone();

            tab.fileName = fullPath.clone();

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
                                       &mut tab.outlineKeywords,
                                       &self.luaSyntaxHighlightScripts
                        ).await
                    }
                );
                lineNumber += 1;
            }
            (tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);

            self.codeTabs.tabs.push(tab);
            self.codeTabs.tabFileNames.push(name.clone());
        }
    }

    async fn HandlePress (&mut self, events: &KeyParser, event: &MouseEvent) {
        if  event.position.0 > 29 &&
            event.position.1 < self.area.height - 8 &&
            event.position.1 > 3 &&
            self.codeTabs.tabs.len() > 0
        {
            self.PressedCode(events, event);
        } else if
        event.position.0 <= 29 &&
            event.position.1 < self.area.height - 10 &&
            matches!(self.fileBrowser.fileTab, FileTabs::Outline) &&
            self.codeTabs.tabs.len() > 0
        {
            self.PressedScopeJump(events, event);
        } else if
        event.position.0 > 29 &&
            event.position.1 <= 2 &&
            self.codeTabs.tabs.len() > 0
        {
            self.PressedNewCodeTab(events, event);
        } else if  // selecting files to open
        event.position.0 > 1 &&
            event.position.0 < 30 && //height - 8, width 30
            event.position.1 < self.area.height - 8 &&
            event.position.1 > 1
        {
            self.PressedLoadFile(events, event).await;
        }
    }

    fn HighlightLeftClick (&mut self, _events: &KeyParser, event: &MouseEvent) {
        // updating the highlighting position
        self.lastTab = self.codeTabs.GetTabNumber(
            &self.area, 29,
            event.position.0 as usize,
            &mut self.lastTab
        );
        // adjusting the position
        let position = (
            self.codeTabs.GetRelativeTabPosition(event.position.0, self.area, 33),
            event.position.1
        );

        let cursorEnding = self.codeTabs.tabs[self.lastTab].cursor;

        let tab = &mut self.codeTabs.tabs[self.lastTab];
        let lineSize = tab.lines.len().to_string().len();  // account for the length of the total lines
        let linePos = (std::cmp::max(tab.scrolled as isize + tab.mouseScrolled, 0) as usize +
                           position.1.saturating_sub(4) as usize,
                       position.0.saturating_sub(lineSize as u16) as usize);
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
            self.codeTabs.tabs[self.lastTab].highlighting = false;
        }
    }

    async fn HandleLeftClick (&mut self, events: &KeyParser, event: &MouseEvent) {
        // checking for code selection
        if matches!(event.state, MouseState::Release | MouseState::Hold) {
            if event.position.0 > 29 && event.position.1 < self.area.height - 8 &&
                event.position.1 > 3 &&
                self.codeTabs.tabs.len() > 0
            {
                self.HighlightLeftClick(events, event);
            }
        } else if matches!(event.state, MouseState::Press) {
            self.HandlePress(events, event).await;
        }
    }

    fn HandleMenuMouseEvents (&mut self, _events: &KeyParser, _event: &MouseEvent) {
        // todo!
    }

    async fn HandleMouseEvents (&mut self, events: &KeyParser) {
        if let Some(event) = &events.mouseEvent {
            if matches!(self.appState, AppState::Menu) {
                self.HandleMenuMouseEvents(events, event);
                return;
            }

            match event.eventType {
                MouseEventType::Down if self.codeTabs.tabs.len() > 0 => {
                    self.HandleDownScroll(event);
                },
                MouseEventType::Up if self.codeTabs.tabs.len() > 0 => {
                    self.HandleUpScroll(event);
                },
                MouseEventType::Left => {
                    self.HandleLeftClick(events, event).await;
                },
                MouseEventType::Middle => {},
                MouseEventType::Right => {},
                _ => {},
            }
        }
    }

    fn HandleCommandPromptKeyEvents (&mut self, keyEvents: &KeyParser) {
        for chr in &keyEvents.charEvents {
            self.currentCommand.push(*chr);
        }

        if keyEvents.ContainsKeyCode(KeyCode::Tab) {
            if keyEvents.ContainsModifier(&KeyModifiers::Shift) &&
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
                self.HandleCodeTabviewKeyEvents(keyEvents);
            },
            TabState::Files => {
                self.HandleFilebrowserKeyEvents(keyEvents);
            },
            TabState::Tabs => {
                self.HandleTabsKeyEvents(keyEvents);
            },
        }

        if !self.currentCommand.is_empty() {
            self.HandleCommands(keyEvents);
        }
    }

    fn JumpLineUp (&mut self) {
        // jumping up
        if let Some(numberString) = self.currentCommand.get(1..) {
            let number = numberString.parse:: <usize>();
            if number.is_ok() {
                let cursor = self.codeTabs.tabs[self.lastTab].cursor.0;
                self.codeTabs.tabs[self.lastTab].JumpCursor(
                    cursor.saturating_sub(number.unwrap()), 1
                );
            }
        }
    }

    fn JumpLineDown (&mut self) {
        // jumping down
        if let Some(numberString) = self.currentCommand.get(1..) {
            let number = numberString.parse:: <usize>();
            if number.is_ok() {
                let cursor = self.codeTabs.tabs[self.lastTab].cursor.0;
                self.codeTabs.tabs[self.lastTab].JumpCursor(
                    cursor.saturating_add(number.unwrap()), 1
                );
            }
        }
    }

    fn HandleCommands (&mut self, keyEvents: &KeyParser) {
        // quiting
        if keyEvents.ContainsKeyCode(KeyCode::Return) {
            if self.currentCommand == "q" {
                self.Exit();
            }

            // jumping to, command
            if self.currentCommand.starts_with('[') {
                self.JumpLineUp();
            } else if self.currentCommand.starts_with(']') {
                self.JumpLineDown();
            } else if self.currentCommand == String::from("gd") {
                // todo!
            }

            self.currentCommand.clear();
        } else if keyEvents.ContainsKeyCode(KeyCode::Delete) {
            self.currentCommand.pop();
        }
    }

    fn HandleCodeTabviewKeyEvents (&mut self, keyEvents: &KeyParser) {
        if keyEvents.ContainsKeyCode(KeyCode::Return) {
            self.appState = AppState::Tabs;
        }
    }

    fn HandleFilebrowserKeyEvents (&mut self, keyEvents: &KeyParser) {
        if matches!(self.fileBrowser.fileTab, FileTabs::Outline) {
            if keyEvents.ContainsKeyCode(KeyCode::Return) && self.currentCommand.is_empty() {
                let mut nodePath = self.codeTabs.tabs[self.lastTab].linearScopes[
                    self.fileBrowser.outlineCursor].clone();
                nodePath.reverse();
                let node = self.codeTabs.tabs[self.lastTab].scopes.GetNode(
                    &mut nodePath
                );
                let start = node.start;
                self.codeTabs.tabs[self.lastTab].JumpCursor(start, 1);
            } else if keyEvents.ContainsKeyCode(KeyCode::Up) {
                self.fileBrowser.MoveCursorUp();
            } else if keyEvents.ContainsKeyCode(KeyCode::Down) {
                self.fileBrowser.MoveCursorDown(
                    &self.codeTabs.tabs[self.lastTab].linearScopes,
                    &self.codeTabs.tabs[self.lastTab].scopes);
            }
        }
    }

    fn HandleTabsKeyEvents (&mut self, keyEvents: &KeyParser) {
        if keyEvents.ContainsKeyCode(KeyCode::Left) {
            if keyEvents.ContainsModifier(&KeyModifiers::Option) {
                self.codeTabs.MoveTabLeft()
            } else {
                self.codeTabs.TabLeft();
            }
        } else if keyEvents.ContainsKeyCode(KeyCode::Right) {
            if keyEvents.ContainsModifier(&KeyModifiers::Option) {
                self.codeTabs.MoveTabRight()
            } else {
                self.codeTabs.TabRight();
            }
        } else if keyEvents.ContainsKeyCode(KeyCode::Return) {
            self.appState = AppState::Tabs;
            self.tabState = TabState::Code;
        } else if keyEvents.ContainsKeyCode(KeyCode::Delete) {
            self.codeTabs.tabs.remove(self.lastTab);
            self.codeTabs.tabFileNames.remove(self.codeTabs.currentTab);
            self.codeTabs.currentTab = self.codeTabs.currentTab.saturating_sub(1);
            if self.lastTab >= self.codeTabs.tabs.len() {
                self.lastTab = self.codeTabs.tabs.len() - 1;
            }
        }
    }

    async fn TypeCode (&mut self, keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        // making sure command + s or other commands aren't being pressed
        if !keyEvents.ContainsModifier(&self.preferredCommandKeybind) {
            for chr in &keyEvents.charEvents {
                if *chr == '(' {
                    self.codeTabs.tabs[self.lastTab]
                        .InsertChars("()".to_string(), &self.luaSyntaxHighlightScripts).await;
                    self.codeTabs.tabs[self.lastTab].cursor.1 -= 1;
                } else if *chr == '{' {
                    self.codeTabs.tabs[self.lastTab]
                        .InsertChars("{}".to_string(), &self.luaSyntaxHighlightScripts).await;
                    self.codeTabs.tabs[self.lastTab].cursor.1 -= 1;
                } else if *chr == '[' {
                    self.codeTabs.tabs[self.lastTab]
                        .InsertChars("[]".to_string(), &self.luaSyntaxHighlightScripts).await;
                    self.codeTabs.tabs[self.lastTab].cursor.1 -= 1;
                } else if *chr == '\"' {
                    self.codeTabs.tabs[self.lastTab]
                        .InsertChars("\"\"".to_string(), &self.luaSyntaxHighlightScripts).await;
                    self.codeTabs.tabs[self.lastTab].cursor.1 -= 1;
                } else {
                    self.codeTabs.tabs[self.lastTab]
                        .InsertChars(chr.to_string(), &self.luaSyntaxHighlightScripts).await;
                }
            }
        }
    }

    async fn DeleteCode (&mut self, keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        let mut numDel = 1;
        let mut offset = 0;

        if keyEvents.keyModifiers.contains(&KeyModifiers::Option) {
            if keyEvents.ContainsModifier(&KeyModifiers::Shift) {
                numDel = self.codeTabs.tabs[self.lastTab].FindTokenPosRight();
                offset = numDel;
            } else {
                numDel = self.codeTabs.tabs[self.lastTab].FindTokenPosLeft();
            }
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) {
            if keyEvents.ContainsModifier(&KeyModifiers::Shift) {
                numDel = self.codeTabs.tabs[self.lastTab].lines[
                    self.codeTabs.tabs[self.lastTab].cursor.0
                    ].len() - self.codeTabs.tabs[self.lastTab].cursor.1;
                offset = numDel;
            } else {
                numDel = self.codeTabs.tabs[self.lastTab].cursor.1;
            }
        } else if keyEvents.ContainsModifier(&KeyModifiers::Shift) {
            offset = numDel;
        }

        self.codeTabs.tabs[
            self.lastTab
        ].DelChars(numDel, offset, &self.luaSyntaxHighlightScripts).await;
    }

    fn CloseCodePane (&mut self) {
        if self.codeTabs.panes.contains(&self.lastTab) {
            self.codeTabs.panes.remove(
                self.codeTabs.panes
                    .iter()
                    .position(|e| e == &self.lastTab)
                    .unwrap()
            );
        }
    }

    fn MoveCodeCursorLeft (&mut self, keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        let highlight= self.HandleHighlightOnCursorMove(keyEvents);

        if keyEvents.ContainsModifier(&KeyModifiers::Option) {
            self.codeTabs.tabs[self.lastTab].MoveCursorLeftToken();
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) {
            self.codeTabs.tabs[self.lastTab].mouseScrolledFlt = 0.0;
            self.codeTabs.tabs[self.lastTab].mouseScrolled = 0;
            // checking if it's the true first value or not
            let mut indentIndex = 0usize;
            let cursorLine = self.codeTabs.tabs[self.lastTab].cursor.0;
            for chr in self.codeTabs.tabs[self.lastTab].lines[cursorLine].chars() {
                if chr != ' ' {
                    break;
                } indentIndex += 1;
            }

            if self.codeTabs.tabs[self.lastTab].cursor.1 <= indentIndex {
                self.codeTabs.tabs[self.lastTab].cursor.1 = 0;
            } else {
                self.codeTabs.tabs[self.lastTab].cursor.1 = indentIndex;
            }
        } else {
            self.codeTabs.tabs[self.lastTab].MoveCursorLeft(1, highlight);
        }
    }

    fn HandleHighlightOnCursorMove (&mut self, keyEvents: &KeyParser) -> bool {
        if keyEvents.ContainsModifier(&KeyModifiers::Shift)
        {
            if !self.codeTabs.tabs[self.lastTab].highlighting {
                self.codeTabs.tabs[self.lastTab].cursorEnd =
                    self.codeTabs.tabs[self.lastTab].cursor;
                self.codeTabs.tabs[self.lastTab].highlighting = true;
            } true
        } else {
            false
        }
    }

    fn MoveCodeCursorRight (&mut self, keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        let highlight = self.HandleHighlightOnCursorMove(keyEvents);

        if keyEvents.ContainsModifier(&KeyModifiers::Option) {
            self.codeTabs.tabs[self.lastTab].MoveCursorRightToken();
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) {
            let tab = &mut self.codeTabs.tabs[self.lastTab];
            tab.scrolled = std::cmp::max(tab.mouseScrolledFlt as isize + tab.scrolled as isize, 0) as usize;
            tab.mouseScrolledFlt = 0.0;
            tab.mouseScrolled = 0;

            let cursorLine = self.codeTabs.tabs[self.lastTab].cursor.0;
            self.codeTabs.tabs[self.lastTab].cursor.1 =
                self.codeTabs.tabs[self.lastTab].lines[cursorLine].len();
        } else {
            self.codeTabs.tabs[self.lastTab].MoveCursorRight(1, highlight);
        }
    }

    fn MoveCodeCursorUp (&mut self, keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        let highlight = self.HandleHighlightOnCursorMove(keyEvents);

        if keyEvents.ContainsModifier(&KeyModifiers::Option) {
            let tab = &mut self.codeTabs.tabs[self.lastTab];
            let mut jumps = tab.scopeJumps[tab.cursor.0].clone();
            jumps.reverse();
            tab.JumpCursor(
                tab.scopes.GetNode(&mut jumps).start, 1
            );
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) {
            let tab = &mut self.codeTabs.tabs[self.lastTab];
            tab.scrolled = std::cmp::max(tab.mouseScrolledFlt as isize + tab.scrolled as isize, 0) as usize;
            tab.mouseScrolledFlt = 0.0;
            tab.mouseScrolled = 0;
            tab.cursor.0 = 0;
        } else {
            self.codeTabs.tabs[self.lastTab].CursorUp(highlight);
        }
    }

    fn MoveCodeCursorDown (&mut self, keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        let highlight = self.HandleHighlightOnCursorMove(keyEvents);

        if keyEvents.ContainsModifier(&KeyModifiers::Option) {
            let tab = &mut self.codeTabs.tabs[self.lastTab];
            let mut jumps = tab.scopeJumps[tab.cursor.0].clone();
            jumps.reverse();
            tab.JumpCursor( tab.scopes.GetNode(&mut jumps).end, 1);
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) {
            let tab = &mut self.codeTabs.tabs[self.lastTab];
            tab.scrolled = std::cmp::max(tab.mouseScrolledFlt as isize + tab.scrolled as isize, 0) as usize;
            tab.mouseScrolledFlt = 0.0;
            tab.mouseScrolled = 0;
            tab.cursor.0 =
                tab.lines.len() - 1;
        } else {
            self.codeTabs.tabs[self.lastTab].CursorDown(highlight);
        }
    }

    async fn HandleCodeTabPress (&mut self, keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        if keyEvents.ContainsModifier(&KeyModifiers::Shift) {
            self.codeTabs.tabs[self.lastTab].UnIndent(&self.luaSyntaxHighlightScripts).await;
        } else {
            if self.suggested.is_empty() || !keyEvents.ContainsModifier(&KeyModifiers::Option) {
                self.codeTabs.tabs[self.lastTab]
                    .InsertChars("    ".to_string(), &self.luaSyntaxHighlightScripts).await;
            } else {
                self.codeTabs.tabs[self.lastTab]
                    .RemoveCurrentToken_NonUpdate();
                self.codeTabs.tabs[self.lastTab]
                    .InsertChars(self.suggested.clone(), &self.luaSyntaxHighlightScripts).await;
            }
        }
    }

    async fn CutCode (&mut self, _keyEvents: &KeyParser, clipBoard: &mut Clipboard) {
        // get the highlighted section of text.... or the line if none
        let tab = &mut self.codeTabs.tabs[self.lastTab];
        let text = tab.GetSelection();
        let _ = clipBoard.set_text(text);

        // clearing the rest of the selection
        if tab.highlighting {
            tab.DelChars(0, 0, &self.luaSyntaxHighlightScripts).await;
        } else {
            tab.lines[tab.cursor.0].clear();
            tab.RecalcTokens(tab.cursor.0, 0, &self.luaSyntaxHighlightScripts).await;
            (tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);
        }
    }

    async fn PasteCodeLoop (&mut self, splitLength: usize, i: usize) {
        let tab = &mut self.codeTabs.tabs[self.lastTab];

        if i < splitLength {
            // why does highlight need to be set to true?????? This makes noooo sense??? I give up
            tab.LineBreakIn(true, &self.luaSyntaxHighlightScripts).await;
            // making sure all actions occur on the same iteration
        }
        if i >0 && i < splitLength {
            if let Some(mut elements) = tab.changeBuffer.pop() {
                while let Some(element) = elements.pop() {
                    let size = tab.changeBuffer.len() - 1;
                    tab.changeBuffer[size].insert(0, element);
                }
            }
        }
    }

    async fn PasteCode (&mut self, _keyEvents: &KeyParser, clipBoard: &mut Clipboard) {
        // pasting in the text
        if let Ok(text) = clipBoard.get_text() {
            let splitText = text.split('\n');
            let splitLength = splitText.clone().count() - 1;
            for (i, line) in splitText.enumerate() {
                if line.is_empty() {  continue;  }
                self.codeTabs.tabs[self.lastTab].InsertChars(
                    line.to_string(), &self.luaSyntaxHighlightScripts
                ).await;
                self.PasteCodeLoop(splitLength, i).await;
            }
        }
    }

    fn FindCodeReferenceLine (&mut self, _keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        // finding the nearest occurrence to the cursor
        let tab = &mut self.codeTabs.tabs[self.lastTab];
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
    }

    async fn HandleUndoRedoCode (&mut self, keyEvents: &KeyParser, _clipBoard: &mut Clipboard) {
        if keyEvents.ContainsModifier(&KeyModifiers::Shift) ||
            keyEvents.charEvents.contains(&'r') {  // common/control + r | z+shift = redo
            self.codeTabs.tabs[self.lastTab].Redo(&self.luaSyntaxHighlightScripts).await;
        } else {
            self.codeTabs.tabs[self.lastTab].Undo(&self.luaSyntaxHighlightScripts).await;
        }
    }

    async fn HandleCodeCommands (&mut self, keyEvents: &KeyParser, clipBoard: &mut Clipboard) {
        if keyEvents.ContainsModifier(&self.preferredCommandKeybind) &&
            keyEvents.ContainsChar('s')
        {
            // saving the program
            self.codeTabs.tabs[self.lastTab].Save();
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) &&
            keyEvents.charEvents.contains(&'f')
        {
            self.FindCodeReferenceLine(keyEvents, clipBoard);
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) &&
            (keyEvents.charEvents.contains(&'z') ||  // command z = undo/redo
                keyEvents.charEvents.contains(&'u') ||  // control/command u = undo
                keyEvents.charEvents.contains(&'r'))  // control/command + r = redo
        {
            self.HandleUndoRedoCode(keyEvents, clipBoard).await;
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) &&
            keyEvents.charEvents.contains(&'c')
        {
            // get the highlighted section of text.... or the line if none
            let text = self.codeTabs.tabs[self.lastTab].GetSelection();
            let _ = clipBoard.set_text(text);
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) &&
            keyEvents.charEvents.contains(&'x')
        {
            self.CutCode(keyEvents, clipBoard).await;
        } else if keyEvents.ContainsModifier(&self.preferredCommandKeybind) &&
            keyEvents.charEvents.contains(&'v')
        {
            self.PasteCode(keyEvents, clipBoard).await;
        }
    }

    async fn HandleCodeKeyEvents (&mut self, keyEvents: &KeyParser, clipBoard: &mut Clipboard) {
        self.TypeCode(keyEvents, clipBoard).await;

        if keyEvents.ContainsKeyCode(KeyCode::Delete) {
            self.DeleteCode(keyEvents, clipBoard).await;
        } else if keyEvents.ContainsChar('w') &&
            keyEvents.ContainsModifier(&KeyModifiers::Option)
        {
            self.CloseCodePane();
        } else if keyEvents.ContainsKeyCode(KeyCode::Left) {
            self.MoveCodeCursorLeft(keyEvents, clipBoard);
        } else if keyEvents.ContainsKeyCode(KeyCode::Right) {
            self.MoveCodeCursorRight(keyEvents, clipBoard);
        } else if keyEvents.ContainsKeyCode(KeyCode::Up) {
            self.MoveCodeCursorUp(keyEvents, clipBoard);
        } else if keyEvents.ContainsKeyCode(KeyCode::Down) {
            self.MoveCodeCursorDown(keyEvents, clipBoard);
        } else if keyEvents.ContainsKeyCode(KeyCode::Tab) {
            self.HandleCodeTabPress(keyEvents, clipBoard).await;
        } else if keyEvents.ContainsKeyCode(KeyCode::Return) {
            self.codeTabs.tabs[self.lastTab].LineBreakIn(false, &self.luaSyntaxHighlightScripts).await;  // can't be highlighting if breaking?
        } else {
            self.HandleCodeCommands(keyEvents, clipBoard).await;
        }
    }

    fn HandleMenuKeyEvents (&mut self, keyEvents: &KeyParser) {
        for chr in &keyEvents.charEvents {
            self.currentCommand.push(*chr);

            if self.currentCommand.starts_with("open ") && *chr == '/' {
                // recalculating the directories
                self.currentDir = FileBrowser::GetPathName(
                    self.currentCommand.get(5..)
                        .unwrap_or("")
                );

                self.dirFiles.clear();
                FileBrowser::CalculateDirectories(&self.currentDir, &mut self.dirFiles);
            }
        }

        if !self.currentCommand.is_empty() {
            self.HandleMenuCommandKeyEvents(keyEvents);
        }

        if matches!(self.menuState, MenuState::Settings) {
            self.HandleSettingsKeyEvents(keyEvents);
        }
    }

    fn HandleMenuCommandKeyEvents (&mut self, keyEvents: &KeyParser) {
        // quiting
        if keyEvents.ContainsKeyCode(KeyCode::Return) {
            if self.currentCommand == "q" {
                self.Exit();
            }

            if self.currentCommand == "settings" {
                self.menuState = MenuState::Settings;
            }

            if self.currentCommand.starts_with("open ") {
                let foundFile = self.fileBrowser
                    .LoadFilePath(self.currentCommand
                                      .get(5..)
                                      .unwrap_or(""), &mut self.codeTabs);

                match foundFile {
                    Ok(_) => {
                        //self.fileBrowser.LoadFilePath("Desktop/Programing/Rust/TermEdit/src/", &mut self.codeTabs);
                        self.fileBrowser.fileCursor = 0;
                        self.codeTabs.currentTab = 0;

                        self.appState = AppState::CommandPrompt;
                    },
                    _ => {},
                }
            }

            self.currentCommand.clear();
        } else if keyEvents.ContainsKeyCode(KeyCode::Delete) {
            self.currentCommand.pop();
        } else if keyEvents.ContainsKeyCode(KeyCode::Tab) &&
            self.currentCommand.starts_with("open ")
        {
            self.OpenCodeProject();
        }
    }

    fn OpenCodeProject(&mut self) {
        // handling suggested directory auto fills
        let currentToken = self.currentCommand
            .split("/")
            .last()
            .unwrap_or("");

        for path in &self.dirFiles {
            if path.starts_with(currentToken) {
                let pathEnding = path
                    .get(currentToken.len()..)
                    .unwrap_or("")
                    .to_string();
                self.currentCommand.push_str(&pathEnding);
                self.currentCommand.push('/');

                // recalculating the directories
                self.currentDir = FileBrowser::GetPathName(
                    self.currentCommand.get(5..)
                        .unwrap_or("")
                );

                self.dirFiles.clear();
                FileBrowser::CalculateDirectories(&self.currentDir, &mut self.dirFiles);
                break;
            }
        }
    }

    fn HandleSettingsLeft (&mut self, _keyEvents: &KeyParser) {
        match self.currentMenuSettingBox {
            0 => {
                self.colorMode.colorType = match self.colorMode.colorType {
                    ColorTypes::BasicColor => ColorTypes::BasicColor,
                    ColorTypes::PartialColor => ColorTypes::BasicColor,
                    ColorTypes::TrueColor => ColorTypes::PartialColor,
                }
            },
            1 => {
                if matches!(self.preferredCommandKeybind, KeyModifiers::Control) {
                    self.preferredCommandKeybind = KeyModifiers::Command;
                }
            }
            _ => {},
        }
    }

    fn HandleSettingsRight (&mut self, _keyEvents: &KeyParser) {
        match self.currentMenuSettingBox {
            0 => {
                self.colorMode.colorType = match self.colorMode.colorType {
                    ColorTypes::BasicColor => ColorTypes::PartialColor,
                    ColorTypes::PartialColor => ColorTypes::TrueColor,
                    ColorTypes::TrueColor => ColorTypes::TrueColor,
                }
            },
            1 => {
                if matches!(self.preferredCommandKeybind, KeyModifiers::Command) {
                    self.preferredCommandKeybind = KeyModifiers::Control;
                }
            }
            _ => {},
        }
    }

    fn HandleSettingsKeyEvents (&mut self, keyEvents: &KeyParser) {
        if keyEvents.ContainsKeyCode(KeyCode::Left) {
            self.HandleSettingsLeft(keyEvents);
        } else if keyEvents.ContainsKeyCode(KeyCode::Right) {
            self.HandleSettingsRight(keyEvents);
        } else if keyEvents.ContainsKeyCode(KeyCode::Up) {
            self.currentMenuSettingBox = self.currentMenuSettingBox.saturating_sub(1);
        } else if keyEvents.ContainsKeyCode(KeyCode::Down) {
            self.currentMenuSettingBox += 1;
        }
    }

    async fn HandleKeyEvents (&mut self, keyEvents: &KeyParser, clipBoard: &mut Clipboard) {
        match self.appState {
            AppState::CommandPrompt => {
                self.HandleCommandPromptKeyEvents(keyEvents);
            },
            AppState::Tabs => {
                match self.tabState {
                    TabState::Code if self.codeTabs.tabs.len() > 0 => {
                        self.HandleCodeKeyEvents(keyEvents, clipBoard).await;
                    },
                    _ => {}  // the other two shouldn't be accessible during the tab state (only during command-line)
                }
            },
            AppState::Menu => {
                self.HandleMenuKeyEvents(keyEvents);
            },
            //_ => {},
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
                _ => {
                    if matches!(self.menuState, MenuState::Settings) {
                        self.menuState = MenuState::Welcome;
                    }

                    AppState::Menu
                },
            }
        }
    }

    fn Exit(&mut self) {
        self.exit = true;
    }

    // ============================================= file block here =============================================
    fn RenderFileBlock (&mut self, area: Rect, buf: &mut Buffer){
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
                height: 3,
            }, buf);
    }

    // ============================================= code block here =============================================
    fn RenderCodeBlock (&mut self, area: Rect, buf: &mut Buffer) {
        if self.codeTabs.tabs.len() > 0 {
            let tabSize = self.codeTabs.GetTabSize(&area, 29);

            for tabIndex in 0..=self.codeTabs.panes.len() {
                self.RenderCodeTab(area, buf, tabIndex, tabSize);
            }
        }
    }

    fn RenderCodeTab(&mut self, area: Rect, buf: &mut Buffer, tabIndex: usize, tabSize: usize) {
        let codeBlockTitle = Line::from(vec![
            " ".to_string().white(),
            self.codeTabs.tabs[
                {
                    if tabIndex == 0 { self.codeTabs.currentTab } else { self.codeTabs.panes[tabIndex - 1] }
                }
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
                    matches!(self.tabState, TabState::Code),
                &self.colorMode,
                &self.suggested,
                {
                    if tabIndex == 0 { self.codeTabs.currentTab } else { self.codeTabs.panes[tabIndex - 1] }
                },
            )
        );

        Paragraph::new(codeText)
            .block(codeBlock)
            .render(Rect {
                x: area.x + 29 + (tabIndex * tabSize) as u16,
                y: area.y + 2,
                width: tabSize as u16,
                //width: area.width - 29,
                height: area.height - 10,
        }, buf);
    }

    fn RenderOutlinePartOne (&self, scopeIndex: &Vec <usize>) -> String {
        let mut offset = String::new();
        if *scopeIndex ==
            self.codeTabs.tabs[self.lastTab]
                .scopeJumps[self.codeTabs.tabs[self.lastTab].cursor.0] &&
            matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) {
            offset.push('>')
        } else if
        matches!(self.appState, AppState::CommandPrompt) &&
            matches!(self.tabState, TabState::Files) &&
            self.codeTabs.tabs[self.lastTab].linearScopes[
                self.fileBrowser.outlineCursor
                ] == *scopeIndex {
            offset.push('>');
        }
        for _ in 0..scopeIndex.len().saturating_sub(1) {
            offset.push_str("  ");
        }
        offset
    }

    fn GetColoredScope <'span> (&self, scopeName: String, scopeLength: usize) -> ratatui::text::Span <'span> {
        match scopeLength {
            1 => scopeName.light_blue(),
            2 => scopeName.light_magenta(),
            3 => scopeName.light_red(),
            4 => scopeName.light_yellow(),
            5 => scopeName.light_green(),
            _ => scopeName.white(),
        }
    }

    fn GetFilebrowserOutline (&self, fileStringText: &mut Vec <Line>, scopeIndex: &Vec <usize>, scope: &ScopeNode) {
        fileStringText.push(
            Line::from(vec![
                {
                    self.RenderOutlinePartOne (scopeIndex).white()
                },
                {
                    // this is a mess...
                    if *scopeIndex ==
                        self.codeTabs.tabs[self.lastTab]
                            .scopeJumps[self.codeTabs.tabs[self.lastTab].cursor.0] &&
                        matches!(self.appState, AppState::CommandPrompt) &&
                        matches!(self.tabState, TabState::Code)
                    {
                        self.GetColoredScope(scope.name.clone(), scopeIndex.len()).underlined()
                    } else if
                        matches!(self.appState, AppState::CommandPrompt) &&
                        matches!(self.tabState, TabState::Files) &&
                        self.codeTabs.tabs[self.lastTab].linearScopes[
                            self.fileBrowser.outlineCursor
                            ] == *scopeIndex
                    {
                        self.GetColoredScope(scope.name.clone(), scopeIndex.len()).underlined()
                    } else {
                        self.GetColoredScope(scope.name.clone(), scopeIndex.len())
                    }
                },
                //format!(" ({}, {})", scope.start + 1, scope.end + 1).white(),  // (not enough space for it to fit...)
            ])
        );
    }

    fn HandleScrolled(&self, scrolled: usize, newScroll: &mut usize, scopeIndex: &Vec <usize>) {
        let tab = &self.codeTabs.tabs[self.lastTab];
        if *scopeIndex == tab.scopeJumps[tab.cursor.0] &&
            matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code)
        {
            *newScroll = scrolled - 1;
        }
    }

    fn RenderFilebrowserOutline (&mut self, area: Rect) -> Text {
        let mut fileStringText = vec!();
        let mut scopes: Vec<usize> = vec![];

        let mut newScroll = self.fileBrowser.outlineCursor;
        let mut scrolled = 0;
        let scrollTo = self.fileBrowser.outlineCursor.saturating_sub(((area.height - 8) / 2) as usize);

        for scopeIndex in &self.codeTabs.tabs[self.lastTab].scopeJumps {
            let mut valid = true;
            for i in 0..scopes.len() {
                let slice = scopes.get(0..(scopes.len() - i));
                if slice.unwrap_or(&[]) != *scopeIndex {  continue;  }
                valid = false;
                break;
            }
            if !valid || scopeIndex.is_empty() {  continue;  }
            scopes.clear();

            let mut scope = &self.codeTabs.tabs[self.lastTab].scopes;
            for index in scopeIndex {
                scopes.push(*index);
                scope = &scope.children[*index];
            }

            scrolled += 1;
            self.HandleScrolled(scrolled, &mut newScroll, &scopeIndex);

            if scrolled < scrollTo { continue; }
            self.GetFilebrowserOutline(&mut fileStringText, &scopeIndex, scope);
        }
        self.fileBrowser.outlineCursor = newScroll;
        Text::from(fileStringText)
    }

    // ============================================= files =============================================
    fn RenderFiles (&mut self, area: Rect, buf: &mut Buffer) {
        let mut fileBlock = Block::bordered()
            .border_set(border::THICK);
        if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Files) {
            fileBlock = fileBlock.light_blue();
        }

        let fileText: Text;

        if matches!(self.fileBrowser.fileTab, FileTabs::Outline) {
            fileText = self.RenderFilebrowserOutline(area);
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
                height: area.height - 8,
            }, buf);
    }

    fn CalculateInnerScope (&mut self, currentScope: &mut Vec <usize>, tokenSet: &mut Vec <String>) {
        if tokenSet.is_empty() {  return;  }

        let mut currentElement = OutlineKeyword::TryFindKeyword(
            &self.codeTabs.tabs[self.lastTab].outlineKeywords,
            tokenSet.pop().unwrap(),
        );
        if let Some(set) = &currentElement {
            let newScope = self.codeTabs.tabs[self.lastTab].scopeJumps[
                set.lineNumber
                ].clone();
            //self.debugInfo.push_str(&format!("{:?} ", newScope.clone()));
            *currentScope = newScope;
        }

        while !tokenSet.is_empty() && currentElement.is_some() {
            //self.debugInfo.push(' ');
            let newToken = tokenSet.remove(0);
            if let Some(set) = currentElement {
                let newScope = self.codeTabs.tabs[self.lastTab].scopeJumps[
                    set.lineNumber
                    ].clone();
                //self.debugInfo.push_str(&format!("{:?} ", newScope.clone()));
                *currentScope = newScope;
                currentElement = OutlineKeyword::TryFindKeyword(&set.childKeywords, newToken);
            }
        }
    }

    fn ParseKeywordSuggestions (&mut self, validKeywords: Vec <OutlineKeyword>, token: String) {
        if matches!(token.as_str(), " " | "," | "|" | "}" | "{" | "[" | "]" | "(" | ")" |
                    "+" | "=" | "-" | "_" | "!" | "?" | "/" | "<" | ">" | "*" | "&" |
                    ".") {  return;  }

        let mut closest = (usize::MAX, vec!["".to_string()], 0usize);
        for (i, var) in validKeywords.iter().enumerate() {
            /*
            if !var.scope.is_empty() {  // matches!(var.kwType, OutlineType::Function) {
                self.debugInfo.push('(');
                self.debugInfo.push_str(var.keyword.as_str());
                //self.debugInfo.push('/');
                //self.debugInfo.push_str(&format!("{:?}", var.scope));
                self.debugInfo.push(')');
            }  // */
            let value = string_pattern_matching::byte_comparison(&token, &var.keyword);  // StringPatternMatching::levenshtein_distance(&token, &var.keyword); (too slow)
            if value < closest.0 {
                closest = (value, vec![var.keyword.clone()], i);
            } else if value == closest.0 {
                closest.1.push(var.keyword.clone());
            }
        }
        // getting the closest option to the size of the current token if there are multiple equal options
        let mut finalBest = (usize::MAX, "".to_string(), 0usize);
        for element in closest.1 {
            let size = (element.len() as isize - token.len() as isize).unsigned_abs();
            if size < finalBest.0 {
                finalBest = (size, element, closest.2);
            }
        }

        if closest.0 < 15 {  // finalBest.1 != token.as_str()
            if token == finalBest.1 {
            } else {
                self.suggested = finalBest.1;
            }
        }
    }

    fn UpdateRenderErrorBar (&mut self, _area: Rect, _buf: &mut Buffer) {
        if self.codeTabs.tabs.is_empty() {  return;  }

        self.suggested.clear();
        //let mut scope = self.codeTabs.tabs[self.lastTab].scopeJumps[
        //    self.codeTabs.tabs[self.lastTab].cursor.0
        //    ].clone();
        let mut tokenSet: Vec<String> = vec!();
        self.codeTabs.tabs[self.lastTab].GetCurrentToken(&mut tokenSet);
        if tokenSet.is_empty() {  return;  }  // token set is correct it seems

        let token = tokenSet.remove(0);  // getting the item actively on the cursor
        // self.debugInfo.push_str("{");
        // self.debugInfo.push_str(&token);
        // self.debugInfo.push_str("}");
        let mut currentScope =
            self.codeTabs.tabs[self.lastTab].scopeJumps[
                self.codeTabs.tabs[self.lastTab].cursor.0
                ].clone();
        self.CalculateInnerScope(&mut currentScope, &mut tokenSet);

        //scope = currentScope.clone();
        let validKeywords = OutlineKeyword::GetValidScoped(
            &self.codeTabs.tabs[self.lastTab].outlineKeywords,
            &currentScope,
        );
        self.ParseKeywordSuggestions(validKeywords, token);
    }

    // ============================================= Error Bar =============================================
    fn RenderErrorBar (&mut self, area: Rect, buf: &mut Buffer) {
        let errorBlock = Block::bordered()
            .border_set(border::THICK);

        // temp todo! replace elsewhere (the sudo auto-checker is kinda crap tbh)
        self.debugInfo.clear();
        /*
        for var in &self.codeTabs.tabs[self.lastTab].outlineKeywords {
            if matches!(var.kwType, OutlineType::Function) {
                self.debugInfo.push('(');
                self.debugInfo.push_str(var.keyword.as_str());
                self.debugInfo.push('/');
                self.debugInfo.push_str(&format!("{:?}", var.scope));
                self.debugInfo.push(')');
            }
        }*/

        self.UpdateRenderErrorBar(area, buf);

        let errorText = Text::from(vec![
            Line::from(vec![
                format!(": {}", self.suggested).fg(
                    self.colorMode.colorBindings.suggestion
                ).italic(),
            ]),
            Line::from(vec![
                format!("Debug: {}", self.debugInfo).fg(
                    self.colorMode.colorBindings.errorCol
                ).bold(),
                //format!(" ; {:?}", scope).white()
            ]),
        ]);

        Paragraph::new(errorText)
            .block(errorBlock)
            .render(Rect {
                x: area.x,
                y: area.y + area.height - 9,
                width: area.width,
                height: 8,
            }, buf);
    }

    fn RenderProject (&mut self, area: Rect, buf: &mut Buffer) {
        self.RenderFileBlock(area, buf);
        self.RenderCodeBlock(area, buf);
        self.RenderFiles(area, buf);
        self.RenderErrorBar(area, buf);
    }

    fn RenderSettings (&mut self, area: Rect, buf: &mut Buffer) {
        // ============================================= Color Settings =============================================
        // the color mode setting
        let settingsText = Text::from(
            vec![
                Line::from(vec![
                    "Color Mode: [".white(),
                    {
                        if matches!(self.colorMode.colorType, ColorTypes::BasicColor) {
                            "Basic".yellow().bold().underlined()
                        } else {
                            "Basic".white()
                        }
                    },
                    "]".white(),
                    " [".white(),
                    {
                        if matches!(self.colorMode.colorType, ColorTypes::PartialColor) {
                            "8-bit".yellow().bold().underlined()
                        } else {
                            "8-bit".white()
                        }
                    },
                    "]".white(),
                    " [".white(),
                    {
                        if matches!(self.colorMode.colorType, ColorTypes::TrueColor) {
                            "24-bit".yellow().bold().underlined()
                        } else {
                            "24-bit".white()
                        }
                    },
                    "]".white(),
                ]),
                Line::from(vec![
                    " * Not all terminals accept all color modes. If the colors are messed up, try lowering this".white().dim().italic()
                ]),
            ]
        );

        let mut colorSettingsBlock = Block::bordered()
            .border_set(border::THICK);
        if self.currentMenuSettingBox == 0 {
            colorSettingsBlock = colorSettingsBlock.light_blue();
        }

        Paragraph::new(settingsText)
            .block(colorSettingsBlock)
            .render(Rect {
                x: 10,//area.x + area.width / 2 - 71 / 2,
                y: 2,//area.y + area.height / 2 - 10,
                width: area.width - 20,
                height: 4
        }, buf);

        // ============================================= Key Settings =============================================
        // the color mode setting
        let settingsText = Text::from(
            vec![
                Line::from(vec![
                    "Preferred Modifier Key: [".white(),
                    {
                        if matches!(self.preferredCommandKeybind, KeyModifiers::Command) {
                            "Command".yellow().bold().underlined()
                        } else {
                            "Command".white()
                        }
                    },
                    "]".white(),
                    " [".white(),
                    {
                        if matches!(self.preferredCommandKeybind, KeyModifiers::Control) {
                            "Control".yellow().bold().underlined()
                        } else {
                            "Control".white()
                        }
                    },
                    "]".white(),
                ]),
                Line::from(vec![
                    " * The preferred modifier key for things like ctrl/cmd 'c'".white().dim().italic()
                ]),
            ]
        );

        let mut colorSettingsBlock = Block::bordered()
            .border_set(border::THICK);
        if self.currentMenuSettingBox == 1 {
            colorSettingsBlock = colorSettingsBlock.light_blue();
        }

        Paragraph::new(settingsText)
            .block(colorSettingsBlock)
            .render(Rect {
                x: 10,//area.x + area.width / 2 - 71 / 2,
                y: 6,//area.y + area.height / 2 - 10,
                width: area.width - 20,
                height: 4
        }, buf);
    }

    fn RenderMenu (&mut self, area: Rect, buf: &mut Buffer) {
        // ============================================= Welcome Text =============================================
        /*

        | welcome!
        |\\            //   .==  ||      _===_    _===_   ||\    /||   .==  ||  |
        | \\          //   ||    ||     //   \\  //   \\  ||\\  //||  //    ||  |
        |  \\  //\\  //    ||--  ||     ||       ||   ||  || \\// ||  ||--      |
        |   \\//  \\//     \\==  ||===  \\___//  \\___//  ||      ||  \\==  []  |

        */

        match self.menuState {
            MenuState::Settings => {
                self.RenderSettings(area, buf);
            },
            MenuState::Welcome => {
                let welcomeText = Text::from(vec![
                    Line::from(vec![
                        "\\\\            //   .==  ||      _===_    _===_   ||\\    /||   .==  ||"
                            .red().bold(),
                    ]),
                    Line::from(vec![
                        " \\\\          //   ||    ||     //   \\\\  //   \\\\  ||\\\\  //||  //    ||"
                            .red().bold(),
                    ]),
                    Line::from(vec![
                        "  \\\\  //\\\\  //    ||--  ||     ||       ||   ||  || \\\\// ||  ||--    "
                            .red().bold(),
                    ]),
                    Line::from(vec![
                        "   \\\\//  \\\\//     \\\\==  ||===  \\\\__=//  \\\\___//  ||      ||  \\\\==  []"
                            .red().bold(),
                    ]),
                    Line::from(vec![]),
                    Line::from(vec![]),
                    Line::from(vec![
                        "The command prompt is bellow (Bottom Left):".white().bold()
                    ]),
                    Line::from(vec![]),
                    Line::from(vec![
                        "Press: <".white().bold().dim(),
                        "q".white().bold().dim().italic().underlined(),
                        "> followed by <".white().bold().dim(),
                        "return".white().bold().dim().italic().underlined(),
                        "> to quit".white().bold().dim(),
                    ]),
                    Line::from(vec![
                        "Type ".white().bold().dim(),
                        "\"open\"".white().bold().dim().italic().underlined(),
                        " followed by the path to the directory".white().bold().dim(),
                    ]),
                    Line::from(vec![]),
                    Line::from(vec![
                        "Type ".white().bold().dim(),
                        "\"settings\"".white().bold().dim().underlined().italic(),
                        " to open settings ( <".white().bold().dim(),
                        "esc".white().bold().dim().italic().underlined(),
                        "> to leave )".white().bold().dim(),
                    ]),
                    //Line::from(vec![
                    //    self.dirFiles.concat().white().bold().dim(),
                    //]),
                ]);

                let welcomeBlock = Block::bordered();

                Paragraph::new(welcomeText)
                    .alignment(Alignment::Center)
                    .block(welcomeBlock)
                    .render(Rect {
                        x: area.x + area.width / 2 - 71 / 2,
                        y: area.y + area.height / 2 - 10,
                        width: 71,
                        height: 15
                }, buf);
            }
        }
    }
}

impl <'a> Widget for &mut App <'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.appState {
            AppState::Tabs | AppState::CommandPrompt => {
                self.RenderProject(area, buf);
            },
            AppState::Menu => {
                self.RenderMenu(area, buf)
            }
        }

        // rendering the command line is necessary for all states
        // ============================================= Commandline =============================================
        let commandText = Text::from(vec![
            Line::from(vec![
                "/".to_string().white().bold(),
                self.currentCommand.clone().white().italic(),
                {
                    if  matches!(self.appState, AppState::Menu) &&
                        self.currentCommand.starts_with("open ")
                    {
                        // handling suggested directory auto fills
                        let currentToken = self.currentCommand
                            .split("/")
                            .last()
                            .unwrap_or("");

                        let mut validFinish = String::new();
                        for path in &self.dirFiles {
                            if path.starts_with(currentToken) {
                                validFinish = path
                                    .get(currentToken.len()..)
                                    .unwrap_or("")
                                    .to_string();
                                break;
                            }
                        }

                        validFinish.white().dim()
                    } else {
                        "".white().dim()
                    }
                },
                {
                    if matches!(self.appState, AppState::CommandPrompt | AppState::Menu) {
                        "_".to_string().white().slow_blink().bold()
                    } else {
                        "".to_string().white()
                    }
                },
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
    gd -> go to definition (experimental)

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



maybe show the outline moving while scrolling?
Add scrolling to the outline
make it so when indenting/unindenting it does it for the entire selection if highlighting
make it so when pressing return, it auto sets to the correct indent level
    same thing for various other actions being set to the correct indent level

Fix the highlighting bug when selecting text above and only partially covering a token; It's not rendering the full token and does weird things...
    It has something to do with the end and starting cursor char positions being aligned on the separate lines
Add double-clicking to highlight a token or line
make it so that highlighting than typing () or {} or [] encloses the text rather than replacing it

command f
    Add the ability to type in custom search terms through the command line
    Add the ability to scroll through the various options
    Currently, detection seems to be working at least for single line terms
    Make it jump the cursor to the terms

make command + x properly shift the text onto a new line when pasted (along w/ being at the correct indent level)

make it so that the outline menu, when opened, is placed at the correct location rather than defaulting to the start until re-entering the code tab

maybe move all the checks for the command key modifier to a single check that then corresponds to other checks beneath it? I'm too lazy rn though

Prevent the program from crashing when touch non-u8 characters (either handle the error, or do something but don't crash)
    Ideally the user can still copy and past them along with placing them and deleting them
    (kind of fixed. You can touch it but not interact)

Fix the bug with the scope outline system where it clicks one cell too high when it hasn't been scrolled yet

Fix the bug with highlighting; when highlighting left and pressing right arrow, it stays at the left
    Highlighting right works fine though, idk why

Allow language highlighting to determine unique characters for comments and multi line comments

make a better system that can store keybindings; the user can make a custom one, or there are two defaults: mac custom, standard
    Kind of did this... kinda not

(complete) make any edits cascade down the file (such as multi line comments) and update those lines until terminated
    (kinda incomplete..... me no want do) determine if the set is incomplete so it doesn't  update the whole file
    !!!Todo! Error! Fix the variable checker/saver to handle when variables are removed from the code, idk how; have fun :( is this working????
    Todo! Fix all the countless errors...... A bunch of junk variables are added and parameters aren't; tuples also aren't handled... :(

Todo! Add the undo-redo change thingy for the replacing chars stuff when auto-filling


make the syntax highlighting use known variable/function syntax types once it's known (before use the single line context).
make the larger context and recalculation of scopes & specific variables be calculated on a thread based on a queue and joined
    once a channel indicates completion. (maybe run this once every second if a change is detected).
only update the terminal screen if a key input is detected.
cut down on cpu usage

make suggested code completions render in-line similar to how the file-directories are done

multi-line parameters on functions/methods aren't correctly read
multi-line comments aren't updated properly when just pressing return

todo!! make it so when too many tabs are open it doesn't just crash and die...

Todo!!! Make it so that when typing, it only recalculates the current token selected rather than the whole line
Add a polling delay for when sampling events to hopefully reduce unnecessary computation and cpu usage

fix the scroll bar rendering and general right most interface rendering when in a split pane

use lua for syntax highlighting and dynamically dispatch the files
    find an efficient way to interface with it without constantly re-compiling or anything
    it'll need it's own thread
    maybe do a joint system to allow instant re-calcs while still giving customization
only update the basic 1-line token syntax highlighting for the token currently selected?

it seems the farthest right panes tend to run slower?

tokio and ratatui seem to be the biggest cpu hogs. It might be worth managing threads on
    this end or even doing custom rendering instead of using ratatui.
    I can't even see really much of anything coming from this project's end of the code
    (using flamegraph)

    why not, just remake it all. Ig I must hate myself or something
    Hopefully it'll be faster, and ig I'll learn much more about async rust

there are some bugs with closing tabs

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


