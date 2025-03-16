// snake case is just bad
#![allow(non_snake_case)]

use std::io;

use crossterm::terminal::enable_raw_mode;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::text::Span;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};



// token / syntax highlighting stuff idk
#[derive(Debug)]
pub enum TokenType {
    Bracket,
    SquirlyBracket,
    Parentheses,
    Variable,
    Member,
    Object,
    Function,
    Method,
    Number,
    Logic,
    Math,
    Assignment,
    Endl,
    Macro,
    Const,
    Barrow,
    Lifetime,  // goo luck figuring this out vs. a regular string.........
    String,
    Comment,
    Null,
    Primative,
}

pub fn GenerateTokens (text: String) -> Vec <(TokenType, String)> {
    // move this to a json file so it can be customized by the user if they so chose to
    let lineBreaks = [
        " ".to_string(),
        "#".to_string(),
        ".".to_string(),
        ",".to_string(),
        "(".to_string(),
        ")".to_string(),
        "[".to_string(),
        "]".to_string(),
        "{".to_string(),
        "}".to_string(),
        ";".to_string(),
        ":".to_string(),
        "!".to_string(),
        "/".to_string(),
        "+".to_string(),
        "-".to_string(),
        "*".to_string(),
        "&".to_string(),
        "'".to_string(),
        "//".to_string(),
        "\"".to_string(),
        "<".to_string(),
        ">".to_string(),
        "<=".to_string(),
        ">=".to_string(),
        "+=".to_string(),
        "-=".to_string(),
        "*=".to_string(),
        "/=".to_string(),
        "==".to_string(),
        "||".to_string(),
        "&&".to_string(),
        "/*".to_string(),
        "*/".to_string(),  // good luck with these.............. (i'll ignor them for now.....)
    ];

    let mut current = "".to_string();
    let mut tokenStrs: Vec <String> = vec!();
    for character in text.as_str().chars() {
        if current.len() > 0 && lineBreaks.contains(&character.to_string()) {
            tokenStrs.push(current.clone());
            current.clear();
            current.push(character);
            tokenStrs.push(current.clone());
            current.clear();
        } else {
            current.push(character);
            let mut valid = false;
            for breaker in &lineBreaks {
                if current.contains(breaker) {
                    valid = true;
                    break;
                }
            }
            if valid {
                tokenStrs.push(current.clone());
                current.clear();
            }
        }
    }
    tokenStrs.push(current);

    let mut isComment = false;
    let mut tokens: Vec <(TokenType, String)> = vec!();
    for (index, strToken) in tokenStrs.iter().enumerate() {
        let emptyString = &"".to_string();
        let nextToken = tokenStrs.get(index + 1).unwrap_or(emptyString).as_str();
        let prevToken = {
            if index > 0 {
                &tokenStrs[index - 1]
            } else {  &"".to_string()  }
        };

        tokens.push((
            match strToken.as_str() {
                "/" if prevToken == "*" => {
                    isComment = false;
                    TokenType::Comment
                }
                "/" if prevToken == "/" => {
                    isComment = true;
                    TokenType::Comment
                },
                "*" if prevToken == "/" => {
                    isComment = true;
                    TokenType::Comment
                },
                _s if isComment => TokenType::Comment,
                " " => TokenType::Null,
                "i32" | "isize" | "i16" | "i8" | "i128" | "i64" |
                    "u32" | "usize" | "u16" | "u8" | "u128" | "u64" | 
                    "f16" | "f32" | "f64" | "f128" | "String" |
                    "str" | "Vec" => TokenType::Primative,
                "[" | "]" => TokenType::Bracket,
                "{" | "}" => TokenType::SquirlyBracket,
                "(" | ")" => TokenType::Parentheses,
                "#" => TokenType::Macro,
                //"" => TokenType::Variable,
                s if s.chars().next().map_or(false, |c| {
                    c.is_ascii_digit()
                }) => TokenType::Number,
                "=" if prevToken == ">" || prevToken == "<" || prevToken == "=" => TokenType::Logic,
                s if (prevToken == "&" && s == "&") || (prevToken == "|" && s == "|") => TokenType::Logic,
                s if (nextToken == "&" && s == "&") || (nextToken == "|" && s == "|") => TokenType::Logic,
                ">" | "<" | "if" | "for" | "while" | "in" | "else" | "false" | "true" | "break" | "loop" => TokenType::Logic,
                "=" if prevToken == "+" || prevToken == "-" || prevToken == "*" || prevToken == "/" => TokenType::Math,
                "=" if nextToken == "+" || nextToken == "-" || nextToken == "*" || nextToken == "/" => TokenType::Math,
                "+" | "-" | "*" | "/" => TokenType::Math,
                "let" | "=" | "use" | "mut" => TokenType::Assignment,
                ";" => TokenType::Endl,
                _s if nextToken == "!" => TokenType::Macro,
                "&" => TokenType::Barrow,
                "'" if nextToken == "a" || nextToken.contains("b") => TokenType::Lifetime,  // this is veryyyyyyy generalized.....
                "a" | "b" if prevToken == "'" && (nextToken == "," || nextToken == ">" || nextToken == " ") => TokenType::Lifetime,
                "\"" | "'" => TokenType::String,
                _s if strToken.to_uppercase() == *strToken => TokenType::Const,
                "enum" | "pub" | "struct" | "impl" | "self" => TokenType::Object,
                _s if prevToken == "." && prevToken[..1].to_uppercase() == prevToken[..1] => TokenType::Method,
                _s if prevToken == "." => TokenType::Member,
                "fn" => TokenType::Function,
                _s if strToken[..1].to_uppercase() == strToken[..1] => TokenType::Function,
                _ => TokenType::Null,
            },
        strToken.clone()));
    }

    tokens
}



// application stuff


#[derive(Debug)]
pub struct CodeTab {
    //
    cursor: (usize, usize),  // line pos, char pos inside line
    lines: Vec <String>,
    lineTokens: Vec <Vec <(TokenType, String)>>,
    scrolled: usize,
    name: String,
}

impl CodeTab {
    pub fn MoveCursorLeft (&mut self, amount: usize) {
        self.cursor = (
            self.cursor.0,
            std::cmp::min(
                self.cursor.1,
                self.lines[self.cursor.0].len()
            ).saturating_sub(amount)
        );
    }

    pub fn MoveCursorRight (&mut self, amount: usize) {
        self.cursor = (
            self.cursor.0,
            self.cursor.1.saturating_add(amount)
        )
    }

    pub fn InsertChars (&mut self, chs: String) {
        let length = self.lines[self.cursor.0]
            .len();
        self.lines[self.cursor.0].insert_str(
            std::cmp::min(
                self.cursor.1,
                length
            ),
            chs.as_str()
        );

        self.cursor = (
            self.cursor.0,
            std::cmp::min(
                self.cursor.1,
                length
            ) + chs.len()
        );

        self.RecalcTokens(self.cursor.0);
    }

    pub fn CursorUp (&mut self) {
        self.cursor = (
            self.cursor.0.saturating_sub(1),
            self.cursor.1
        );
    }

    pub fn CursorDown (&mut self) {
        self.cursor = (
            std::cmp::min(
                self.cursor.0.saturating_add(1),
                self.lines.len() - 1
            ),
            self.cursor.1
        );
    }

    pub fn LineBreakBefore (&mut self) {  // correctly deal with tokens here......
        self.lines.insert(
            self.cursor.0,
            "".to_string(),
        );
    }

    pub fn LineBreakIn (&mut self) {
        let length = self.lines[self.cursor.0].len();

        if length == 0 {
            self.lines.insert(self.cursor.0, "".to_string());
            self.lineTokens[self.cursor.0].clear();
            self.lineTokens.insert(self.cursor.0, vec!());
            self.cursor.1 = 0;
            self.CursorDown();
            return;
        }

        let rightSide = self.lines[self.cursor.0]
            .split_off(std::cmp::min(
                self.cursor.1,
            length
        ));

        /*  what was this even for??????
        if self.cursor.0 >= length {
            self.lines.push(rightSide);
            //self.lineTokens[]
            
            self.cursor.1 = 0;
            self.CursorDown();
            return;
        }*/

        self.lines.insert(
            self.cursor.0 + 1,
            rightSide,
        );
        self.lineTokens[self.cursor.0].clear();
        self.lineTokens.insert(
            self.cursor.0 + 1,
            vec!(),
        );
        self.RecalcTokens(self.cursor.0);
        self.RecalcTokens(self.cursor.0 + 1);
        self.cursor.1 = 0;
        self.CursorDown();
    }

    pub fn LineBreakAfter (&mut self) {  // correctly deal with tokens here......
        if self.cursor.0 >= self.lines[self.cursor.0].len() {
            self.lines.push("".to_string());
            self.cursor.1 = 0;
            self.CursorDown();
            return;
        }
        self.lines.insert(
            self.cursor.0 + 1,
            "".to_string(),
        );
        self.cursor.1 = 0;
        self.CursorDown();
    }

    // cursorOffset can be used to delete in multiple directions
    // if the cursorOffset is equal to numDel, it'll delete to the right
    // cursorOffset = 0 is default and dels to the left
    pub fn DelChars (&mut self, numDel: usize, cursorOffset: usize) {
        let length = self.lines[self.cursor.0]
            .len();

        if self.cursor.1 < numDel && cursorOffset == 0 && self.lines.len() > 1 {
            // the remaining text
            let remaining = self.lines[self.cursor.0].split_off(self.cursor.1);

            self.lines.remove(self.cursor.0);
            self.lineTokens.remove(self.cursor.0);
            self.cursor.0 = self.cursor.0.saturating_sub(1);
            self.cursor.1 = self.lines[self.cursor.0].len();

            self.lines[self.cursor.0].push_str(remaining.as_str());
            self.RecalcTokens(self.cursor.0);
            return;
        }
        
        self.cursor = (
            self.cursor.0,
            std::cmp::min(
                self.cursor.1,
                length
            )
        );
        
        self.lines[self.cursor.0]
            .replace_range(
                self.cursor.1
                    .saturating_sub(numDel)
                    .saturating_add(cursorOffset)
                    ..
                    std::cmp::min(
                        self.cursor.1.saturating_add(cursorOffset),
                        length
                ),
                ""
            );
        self.cursor = (
            self.cursor.0,
            self.cursor.1.saturating_sub(1)
        );

        self.RecalcTokens(self.cursor.0);
    }

    pub fn RecalcTokens (&mut self, lineNumber: usize) {
        self.lineTokens[lineNumber].clear();

        let newTokens = GenerateTokens(self.lines[lineNumber].clone());
        self.lineTokens[lineNumber] = newTokens;
    }

    pub fn GenerateColor <'a> (&self, token: &TokenType, text: &'a str) -> Span <'a> {
        match token {
            TokenType::Bracket => {
                text.light_blue()
            },
            TokenType::SquirlyBracket => {
                text.magenta()
            },
            TokenType::Parentheses => {
                text.magenta()
            },
            TokenType::Variable => {
                text.white()
            },
            TokenType::Member => {
                text.light_cyan()
            },
            TokenType::Object => {
                text.light_cyan()
            },
            TokenType::Function => {
                text.light_magenta()
            },
            TokenType::Method => {
                text.light_cyan()
            },
            TokenType::Number => {
                text.light_yellow()
            },
            TokenType::Logic => {
                text.light_yellow()
            },
            TokenType::Math => {
                text.light_yellow()
            },
            TokenType::Assignment => {
                text.light_blue()
            },
            TokenType::Endl => {
                text.white()
            },
            TokenType::Macro => {
                text.blue()
            },
            TokenType::Const => {
                text.cyan()
            },
            TokenType::Barrow => {
                text.white()
            },
            TokenType::Lifetime => {
                text.light_blue()
            },
            TokenType::String => {
                text.yellow()
            },
            TokenType::Comment => {
                text.light_green()
            },
            TokenType::Null => {
                text.white()
            },
            TokenType::Primative => {
                text.light_yellow()
            },
        }
    } 

    pub fn GetScrolledText (&self, area: Rect, editingCode: bool) -> Vec <ratatui::text::Line> {
        let mut tabText = vec![];

        for lineNumber in self.scrolled..(self.scrolled + area.height as usize - 10) {
            if lineNumber >= self.lines.len() {  continue;  }

            /*let mut currentLineLeft = self.lines.get(lineNumber)
                .unwrap_or(&"".to_string())
                    .clone();
            let mut currentLineRight = ""
                .to_string();
            
            if lineNumber == self.cursor.0 {
                currentLineRight = currentLineLeft.split_off(std::cmp::min(self.cursor.1, currentLineLeft.len()));
            }*/

            let mut lineNumberText = format!("{}: ", (lineNumber as isize - self.cursor.0 as isize).unsigned_abs());
            if self.cursor.0 == lineNumber {
                lineNumberText = format!("{}: ", lineNumber+1);
            }
            if lineNumberText.len() == 3 {
                lineNumberText.push(' ');
            }

            let mut coloredLeft: Vec <Span> = vec!();
            let mut coloredRight: Vec <Span> = vec!();

            if lineNumber == self.cursor.0 {
                coloredLeft.push(lineNumberText.red().bold().add_modifier(Modifier::UNDERLINED));
            } else {
                coloredLeft.push(lineNumberText.white().italic());
            }

            let mut currentCharNum = 0;
            for (token, text) in &self.lineTokens[lineNumber] {
                if lineNumber == self.cursor.0 && currentCharNum + text.len() > self.cursor.1 {
                    if currentCharNum >= self.cursor.1 {
                        if currentCharNum == self.cursor.1 && editingCode {
                            coloredLeft.push("|".to_string().white().bold());
                        }
                        coloredRight.push(self.GenerateColor(token, text.as_str()));
                    } else {
                        coloredLeft.push(
                            self.GenerateColor(token,
                            &text[0..text.len() - (
                                currentCharNum + text.len() - self.cursor.1
                            )])
                        );
                        if editingCode {  coloredLeft.push("|".to_string().white().bold())  };
                        coloredRight.push(
                            self.GenerateColor(token,
                            &text[
                                text.len() - (
                                    currentCharNum + text.len() - self.cursor.1
                                )..text.len()
                            ])
                        );
                    }
                } else {
                    coloredLeft.push(self.GenerateColor(token, text.as_str()));
                    //coloredLeft.push("|".to_string().white());  // shows the individual tokens
                }

                currentCharNum += text.len();
            }
            if lineNumber == self.cursor.0 && currentCharNum <= self.cursor.1 && editingCode {
                coloredLeft.push("|".to_string().white().bold());
            }

            let mut finalColText: Vec <Span> = vec!();
            for col in coloredLeft {
                finalColText.push(col);
            } for col in coloredRight {
                finalColText.push(col);
            }

            if self.cursor.0 == lineNumber && editingCode{
                tabText.push(Line::from(
                    finalColText
                ).add_modifier(Modifier::UNDERLINED));
                /*tabText.push(Line::from(vec![
                    lineNumberText.red().bold().add_modifier(Modifier::UNDERLINED),
                    currentLineLeft.white().add_modifier(Modifier::UNDERLINED),
                    "|".to_string().white().bold().slow_blink().add_modifier(Modifier::UNDERLINED),
                    currentLineRight.white().add_modifier(Modifier::UNDERLINED),
                    {
                        let mut string = "".to_string();
                        for _ in 27+(self.lines[self.cursor.0].len() as u16)..area.width {
                            string.push(' ');
                        }
                        string
                    }.add_modifier(Modifier::UNDERLINED),
                ]));*/
            } else {
                tabText.push(Line::from(
                    finalColText
                ));
                /*tabText.push(Line::from(vec![
                    lineNumberText.white().italic(),
                    currentLineLeft.white(),
                    currentLineRight.white(),
                ]));*/
            }
        }

        tabText
    }
}

impl Default for CodeTab {
    fn default() -> Self {
         CodeTab{
            cursor: (0, 0),
            lines: vec![],
            lineTokens: vec!(),
            scrolled: 0,
            name: "Welcome.txt"
                .to_string(),
        }
    }
}


#[derive(Debug)]
pub struct CodeTabs {
    tabs: Vec <CodeTab>,
    currentTab: usize,
}

impl CodeTabs {
    pub fn GetScrolledText (&self, area: Rect, editingCode: bool) -> Vec <ratatui::text::Line> {
        self.tabs[self.currentTab].GetScrolledText(area, editingCode)
    }
}

impl Default for CodeTabs {
    fn default() -> Self {
         CodeTabs {
            tabs: vec![
                CodeTab {
                    cursor: (0, 0),
                    lines: vec![
                        "Welcome! Please open or create a file...".to_string()
                    ],
                    lineTokens: vec![
                        GenerateTokens("Welcome! Please open or create a file...".to_string())
                    ],
                    scrolled: 0,
                    name: "Welcome.txt".to_string(),
                }
            ],  // put a tab here or something idk
            currentTab: 0
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
pub struct KeyModifs {
    Shift: bool,
    Command: bool,
    Option: bool,
    Control: bool,
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    appState: AppState,
    tabState: TabState,
    codeTabs: CodeTabs,
    currentCommand: String,
    //keyModifiers: KeyModifs,
    //escapeSeq: String,
}


impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.HandleEvents()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn HandleEvents(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.HandleKeyEvent(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn HandleKeyEvent(&mut self, key_event: KeyEvent) {
        /*match key_event.code {
            KeyCode::Esc => {
                self.escapeSeq.clear();
                self.escapeSeq.push('\x1B');
            }
            KeyCode::Char(character) => {
                self.escapeSeq.push(character);
            },
            _ => {}
        }  // \x1B[3;2~*/
        //println!(" | {} |", self.escapeSeq);

        //let mut validSeq = false;
        //if self.escapeSeq.contains(&"\x1B[3;2~".to_string()) {  validSeq = true;  }

        match key_event.code {
            KeyCode::Enter if matches!(self.appState, AppState::CommandPrompt) => {
                if self.currentCommand == "q".to_string() {
                    self.Exit();
                }

                self.currentCommand = "".to_string();
            }
            KeyCode::Left if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorLeft(1),
            KeyCode::Right if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorRight(1),
            KeyCode::Backspace => {
                if matches!(self.appState, AppState::CommandPrompt) {
                    self.currentCommand.pop();
                } else if matches!(self.tabState, TabState::Code) {
                    //let mut offset = 0;
                    //if self.escapeSeq.contains(&"\x1B[3;2~".to_string()) {
                        //offset = 1;
                        //println!("YAYYYYAYYAYYAYAYAYAY\nYAYAYAYAYAY");
                    //}
                    //println!("{:?}", key_event);

                    //println!("WOWOOWOWOWO");

                    self.codeTabs.tabs[
                        self.codeTabs.currentTab
                    ].DelChars(1, 0);
                }
            }
            KeyCode::Esc => {
                self.appState = match self.appState {
                    AppState::Tabs => AppState::CommandPrompt,
                    AppState::CommandPrompt => {
                        //self.currentCommand = "".to_string();
                        AppState::Tabs
                    },
                }
            }
            KeyCode::Tab => {
                if matches!(self.appState, AppState::Tabs) {
                    // inside the code editor
                    self.codeTabs.tabs[self.codeTabs.currentTab]
                        .InsertChars("    ".to_string());
                } else {
                    // switching between tabs
                    self.tabState = match self.tabState {
                        TabState::Code => TabState::Files,
                        TabState::Files => TabState::Tabs,
                        TabState::Tabs => TabState::Code,
                    }
                }
            }
            KeyCode::Down if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {
                self.codeTabs.tabs[self.codeTabs.currentTab].CursorDown();
            }
            KeyCode::Up if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {
                self.codeTabs.tabs[self.codeTabs.currentTab].CursorUp();
            }
            KeyCode::Enter if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {
                if key_event.modifiers.contains(KeyModifiers::SUPER) {  // command; alt == option; super == command
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.codeTabs.tabs[self.codeTabs.currentTab].LineBreakBefore();
                    } else {
                        self.codeTabs.tabs[self.codeTabs.currentTab].LineBreakAfter();
                    }
                } else {
                    self.codeTabs.tabs[self.codeTabs.currentTab].LineBreakIn();
                }
            }
            KeyCode::Char(to_insert) if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {// && !validSeq => {
                self.codeTabs.tabs[self.codeTabs.currentTab]
                    .InsertChars(to_insert.to_string());
            }
            KeyCode::Char(to_insert) if matches!(self.appState, AppState::CommandPrompt) => {// && !validSeq => {
                self.currentCommand.push(to_insert);
            }
            _ => {}
        }

        //if validSeq {  self.escapeSeq.clear();  }
    }

    fn Exit(&mut self) {
        self.exit = true;
    }

}


impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
        // ============================================= file block here =============================================
        let tabBlock = Block::bordered()
            .border_set(border::THICK);

        let tabText = Text::from(vec![
            Line::from(vec![
                "(1)".to_string().yellow(),
                " Testing.rs |".to_string().white(),
                "(2)".to_string().yellow(),
                " main.rs |".to_string().white(),
                "(3)".to_string().yellow(),
                " whyIsThisHere.rs ".to_string().white(),
            ])
        ]);

        Paragraph::new(tabText)
            .block(tabBlock)
            .render(Rect {
                x: area.x + 20,
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
        let codeBlock = Block::bordered()
            .title_top(codeBlockTitle.centered())
            .border_set(border::THICK);

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
                x: area.x + 20,
                y: area.y + 2,
                width: area.width - 20,
                height: area.height - 10
        }, buf);


        // ============================================= files =============================================
        let fileBlock = Block::bordered()
            .border_set(border::THICK);

        let fileText = Text::from(vec![
            Line::from(vec![
                "Testing.rs".to_string().white()
            ]),
            Line::from(vec![
                "main.rs".to_string().white()
            ]),
            Line::from(vec![
                "whyIsThisHere.rs".to_string().white()
            ]),
            Line::from(vec![
                "Failed.rs".to_string().white()
            ])
        ]);

        Paragraph::new(fileText)
            .block(fileBlock)
            .render(Rect {
                x: area.x,
                y: area.y,
                width: 21,
                height: area.height - 8
        }, buf);

            
        // ============================================= Error Bar =============================================
        let errorBlock = Block::bordered()
            .border_set(border::THICK);
        
        let errorText = Text::from(vec![
            Line::from(vec![
                "Error: callback on line 5".to_string().red().bold()
            ])
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
Commands: <esc>
    <enter> -> exit commands
    <q> + <enter> -> exit application
    <tab> -> switch tabs:

        * code editor:
            <cmd> + <1 - 9> -> switch to corresponding code tab
            
            <left/right/up/down> -> movement in the open file
            <option> + <left/right> -> jump to next token
            <option> + <up/down> -> jump to end/start of current scope
            <cmnd> + <left/right> -> jump to start/end of line
                - first left jumps to the indented start
                - second left jumps to the true start
            <cmnd> + <up/down> -> jump to start/end of file
            <shift> + any line movement/jump -> highlight all text selected (including jumps from line # inputs)
            <ctrl> + <[> -> temporarily opens command line to input number of lines to jump up
            <ctrl> + <]> -> temporarily opens command line to input number of lines to jump down
            <ctrl> + <1-9> -> jump up that many positions
            <ctrl> + <1-9> + <shift> -> jump down that many positions

            <ctrl> + <left>/<right> -> open/close scope of code (can be done from any section inside the scope, not just the start/end)

            <cmnd> + <c> -> copy (either selection or whole line when none)
            <cmnd> + <v> -> paste to current level (align relative indentation to the cursor's)

            <del> -> deletes the character the cursor is on
            <del> + <option> -> delete the token the cursor is on
            <del> + <cmnd> -> delete the entire line
            <del> + <shift> + <cmnd/option/none> -> does the same as specified before execpt to the right instead

            <tab> -> indents the line up to the predicted indentation
            <tab> + <shift> -> unindents the line by one
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
            
            <shift> + <tab> -> cycle between pg outline and file broswer

            outline:
                - shows all functions/methods/classes/etc... so they can easily be acsess without needed the mouse and without wasting time scrolling

                <enter> -> jumps the cursor to that section in the code
                <shift> + <enter> -> jumps the cursor to the section and switches to code editing
                
                <down/up> -> moves down or up one
                <option> + <down/up> -> moves up or down to the start/end of the current scope
                <cmnd> + <down>/<up> -> moves to the top or bottom of the outline
                
                <ctrl> + <left> -> collapse the scope currently selected
                <ctrl> + <right> -> uncollapse the scope

        * code tabs:
            <left/right> -> change tab
            <option> + <left/right> -> move current tab left/right
            <del> -> close current tab

the bottom bar is 4 lines tall (maybe this could be a custom parameter?)
the side bar appears behind the bottom bar and pops outward shifting the text

Errors underlined in red
Warnings underlined in yellow
    integrate a way to run clippy on the typed code
    parse clippy's output into the proper warning or errors
    display the error or warning at the bottom (where code completion suggestions go)

Suggestions appear on the very bottom as to not obstruct the code being written
*/

fn main() -> io::Result<()> {
    //println!("\x1B[>4;1m");
    enable_raw_mode()?;

    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

