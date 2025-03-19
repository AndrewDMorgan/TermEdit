// snake case is just bad
#![allow(non_snake_case)]

use tokio::io::{self, AsyncReadExt};
use vte::{Parser, Perform};
//use std::io;

use crossterm::terminal::enable_raw_mode;

//use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
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


// the bounds from the screen edge at which the cursor will begin scrolling
const SCROLL_BOUNDS: usize = 12;
const CENTER_BOUNDS: usize = 0;



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
    Keyword,
}

#[derive(Clone)]
pub enum TokenFlags {
    Comment,  // has priority of everything including strings which overrule everything else
    String,  // has priority over everything (including chars)
    Char,  // has 2nd priority (over rules generics)
    Generic,
    Null,
}

pub fn GenerateTokens (text: String) -> Vec <(TokenType, String)> {
    // move this to a json file so it can be customized by the user if they so chose to
    let lineBreaks = [
        " ".to_string(),
        "|".to_string(),
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
        "\"".to_string(),
        "<".to_string(),
        ">".to_string(),
    ];

    let mut current = "".to_string();
    let mut tokenStrs: Vec <String> = vec!();
    for character in text.as_str().chars() {
        if !current.is_empty() && lineBreaks.contains(&character.to_string()) {
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

    // getting any necessary flags
    let mut flags: Vec <TokenFlags> = vec!();
    let mut currentFlag = TokenFlags::Null;  // the current flag being tracked
    for (index, token) in tokenStrs.iter().enumerate() {
        let emptyString = &"".to_string();
        let nextToken = tokenStrs.get(index + 1).unwrap_or(emptyString).as_str();
        
        let newFlag =
            match token.as_str() {
                "/" if nextToken == "/" || nextToken == "*" => TokenFlags::Comment,
                "*" if nextToken == "/" => TokenFlags::Null,
                "\"" if !matches!(currentFlag, TokenFlags::Comment) => {
                    if matches!(currentFlag, TokenFlags::String) {  TokenFlags::Null}
                    else {  TokenFlags::String}
                },
                "<" if !matches!(currentFlag, TokenFlags::Comment | TokenFlags::String | TokenFlags::Char) => TokenFlags::Generic,
                //">" if matches!(currentFlag, TokenFlags::Generic) => TokenFlags::Null,  (unfortunately the lifetimes are needed... not sure how to fix it properly without tracking more info)
                "'" if !matches!(currentFlag, TokenFlags::Comment | TokenFlags::String | TokenFlags::Generic) => {
                    if matches!(currentFlag, TokenFlags::Char) {  TokenFlags::Null}
                    else {  TokenFlags::Char  }
                },
                _ => currentFlag,
        };

        currentFlag = newFlag;
        flags.push(currentFlag.clone());
    }
    
    // track strings and track chars and generic inputs (this might allow for detecting lifetimes properly)
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
                _s if matches!(flags[index], TokenFlags::Comment) => TokenType::Comment,
                _s if matches!(flags[index], TokenFlags::String | TokenFlags::Char) => TokenType::String,
                "if" | "for" | "while" | "in" | "else" |
                    "break" | "loop" | "match" | "return" | "std" |
                    "const" | "static" | "dyn" | "type" | "continue" |
                    "use" | "mod" | "None" | "Some" | "Ok" | "Err" => TokenType::Keyword,
                " " => TokenType::Null,
                "i32" | "isize" | "i16" | "i8" | "i128" | "i64" |
                    "u32" | "usize" | "u16" | "u8" | "u128" | "u64" | 
                    "f16" | "f32" | "f64" | "f128" | "String" |
                    "str" | "Vec" | "bool" | "char" | "Result" |
                    "Option" => TokenType::Primative,
                "[" | "]" => TokenType::Bracket,
                "{" | "}" | "|" => TokenType::SquirlyBracket,
                "(" | ")" => TokenType::Parentheses,
                "#" => TokenType::Macro,
                _s if nextToken == "!" => TokenType::Macro,
                //"" => TokenType::Variable,
                s if s.chars().next().map_or(false, |c| {
                    c.is_ascii_digit()
                }) => TokenType::Number,
                "=" if prevToken == ">" || prevToken == "<" || prevToken == "=" => TokenType::Logic,
                s if (prevToken == "&" && s == "&") || (prevToken == "|" && s == "|") => TokenType::Logic,
                s if (nextToken == "&" && s == "&") || (nextToken == "|" && s == "|") => TokenType::Logic,
                ">" | "<" | "false" | "true" => TokenType::Logic,
                "=" if prevToken == "+" || prevToken == "-" || prevToken == "*" || prevToken == "/" => TokenType::Math,
                "=" if nextToken == "+" || nextToken == "-" || nextToken == "*" || nextToken == "/" => TokenType::Math,
                "+" | "-" | "*" | "/" => TokenType::Math,
                "let" | "=" | "mut" => TokenType::Assignment,
                ";" => TokenType::Endl,
                "&" => TokenType::Barrow,
                "'" if matches!(flags[index], TokenFlags::Generic) => TokenType::Lifetime,
                _s if matches!(flags[index], TokenFlags::Generic) && prevToken == "'" => TokenType::Lifetime,
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
pub struct ScopeNode {
    children: Vec <ScopeNode>,
    name: String,
    start: usize,
    end: usize,
}

impl ScopeNode {
    pub fn GetNode (&self, scope: &mut Vec <usize>) -> &ScopeNode {
        let index = scope.pop();

        if index.is_none() {
            return self;
        }

        /*if index.unwrap() >= self.children.len() {
            return self;
        }*/

        self.children[index.unwrap()].GetNode(scope)
    }

    pub fn Push (&mut self, scope: &mut Vec <usize>, name: String, start: usize) -> usize {
        let index = scope.pop();
        if index.is_none() {
            self.children.push(
                ScopeNode {
                    children: vec![],
                    name,
                    start,
                    end: 0
                }
            );
            return self.children.len() - 1;
        }
        
        self.children[index.unwrap()].Push(scope, name, start)
    }

    pub fn SetEnd (&mut self, scope: &mut Vec <usize>, end: usize) {
        let index = scope.pop();
        if index.is_none() {
            self.end = end;
            return;
        }

        self.children[index.unwrap()].SetEnd(scope, end);
    }
}

const VALID_NAMES_NEXT: [&str; 4] = [
    "fn",
    "struct",
    "enum",
    "impl",
];

const VALID_NAMES_TAKE: [&str; 6] = [
    "for",
    "while",
    "if",
    "else",
    "match",
    "loop",
];

pub fn GenerateScopes (tokenLines: &Vec <Vec <(TokenType, String)>>) -> (ScopeNode, Vec <Vec <usize>>, Vec <Vec <usize>>) {
    // tracking the scope (functions = new scope; struct/enums = new scope; for/while = new scope)
    let mut rootNode = ScopeNode {
        children: vec![],
        name: "Root".to_string(),
        start: 0,
        end: tokenLines.len().saturating_sub(1),
    };

    let mut jumps: Vec <Vec <usize>> = vec!();
    let mut linearized: Vec <Vec <usize>> = vec!();

    let mut currentScope: Vec <usize> = vec!();
    for (lineNumber, tokens) in tokenLines.iter().enumerate() {
        let mut bracketDepth = 0isize;
        // track the depth; if odd than based on the type of bracket add scope; if even do nothing (scope opened and closed on the same line)
        // use the same on functions to determin if the scope needs to continue or end on that line
        for (index, (token, name)) in tokens.iter().enumerate() {
            // checking bracket depth
            if matches!(token, TokenType::SquirlyBracket) {
                if name == "{" {
                    bracketDepth += 1;
                } else {
                    bracketDepth -= 1;
                }
                continue;
            }

            // checking for something to define the name
            if matches!(token, TokenType::Keyword | TokenType::Object | TokenType::Function) {
                if !(VALID_NAMES_NEXT.contains(&name.trim()) || VALID_NAMES_TAKE.contains(&name.trim())) {
                    continue;
                }

                // check bracketdepth here so any overlapping marks are accounted for
                if bracketDepth < 0 {  // not pushing any jumps bc/ it'll be done later
                    let mut scopeCopy = currentScope.clone();
                    scopeCopy.reverse();
                    rootNode.SetEnd(&mut scopeCopy, lineNumber);
                    currentScope.pop();
                }
                
                // checking the scope to see if ti continues or ends on the same line
                let mut brackDepth = 0isize;
                for (indx, (token, name)) in tokens.iter().enumerate() {
                    if indx > index && matches!(token, TokenType::SquirlyBracket) {
                        if name == "{" {
                            brackDepth += 1;
                        } else {
                            brackDepth -= 1;
                        }
                    }
                }
                
                // adding the new scope if necessary
                if brackDepth > 0 {
                    if VALID_NAMES_NEXT.contains(&name.trim()) {
                        let nextName = tokens.get(index + 2).unwrap_or(&(TokenType::Null, "".to_string())).1.clone();
                        
                        let mut scopeCopy = currentScope.clone();
                        scopeCopy.reverse();
                        let mut goodName = name.clone();
                        goodName.push(' ');
                        goodName.push_str(nextName.as_str());
                        let newScope = rootNode.Push(&mut scopeCopy, goodName, lineNumber);
                        currentScope.push(newScope);
                        linearized.push(currentScope.clone());
                        
                        bracketDepth = 0;
                        break;
                    } else if VALID_NAMES_TAKE.contains(&name.trim()) {
                        let mut scopeCopy = currentScope.clone();
                        scopeCopy.reverse();
                        let goodName = name.clone();
                        let newScope = rootNode.Push(&mut scopeCopy, goodName, lineNumber);
                        currentScope.push(newScope);
                        linearized.push(currentScope.clone());

                        bracketDepth = 0;
                        break;
                    }
                }
            }
        }
        
        // updating the scope based on the brackets
        if bracketDepth > 0 {
            let mut scopeCopy = currentScope.clone();
            scopeCopy.reverse();
            let newScope = rootNode.Push(&mut scopeCopy, "{ ... }".to_string(), lineNumber);
            currentScope.push(newScope);
            linearized.push(currentScope.clone());
            jumps.push(currentScope.clone());
        } else if bracketDepth < 0 {
            jumps.push(currentScope.clone());
            let mut scopeCopy = currentScope.clone();
            scopeCopy.reverse();
            rootNode.SetEnd(&mut scopeCopy, lineNumber);
            currentScope.pop();
        } else {
            jumps.push(currentScope.clone());
        }
    }

    (rootNode, jumps, linearized)
}

#[derive(Debug)]
pub struct CodeTab {
    cursor: (usize, usize),  // line pos, char pos inside line
    lines: Vec <String>,
    lineTokens: Vec <Vec <(TokenType, String)>>,
    scopeJumps: Vec <Vec <usize>>,  // points to the index of the scope (needs adjusting as the tree is modified)
    scopes: ScopeNode,
    linearScopes: Vec <Vec <usize>>,
    scrolled: usize,
    name: String,
}

impl CodeTab {

    pub fn MoveCursorLeftToken (&mut self) {
        self.cursor.1 = std::cmp::min (
            self.cursor.1,
            self.lines[self.cursor.0].len()
        );
        
        // walking back till no longer on a space
        while self.cursor.1 > 0 && self.lines[self.cursor.0].get(self.cursor.1-1..self.cursor.1).unwrap_or("") == " " {
            self.cursor.1 -= 1;
        }
        
        let mut totalLine = String::new();
        for (_token, name) in &self.lineTokens[self.cursor.0] {
            if totalLine.len() + name.len() >= self.cursor.1 {
                self.cursor.1 = totalLine.len();
                return;
            }
            totalLine.push_str(name);
        }
    }

    pub fn FindTokenPosLeft (&mut self) -> usize {
        self.cursor.1 = std::cmp::min (
            self.cursor.1,
            self.lines[self.cursor.0].len()
        );
        let mut newCursor = self.cursor.1;

        while newCursor > 0 && self.lines[self.cursor.0].get(newCursor-1..newCursor).unwrap_or("") == " " {
            newCursor -= 1;
        }

        let mut totalLine = String::new();
        for (_token, name) in &self.lineTokens[self.cursor.0] {
            if totalLine.len() + name.len() >= newCursor {
                newCursor = totalLine.len();
                break;
            }
            totalLine.push_str(name);
        }

        self.cursor.1 - newCursor
    }
    
    pub fn MoveCursorRightToken (&mut self) {
        let mut totalLine = String::new();
        for (_token, name) in &self.lineTokens[self.cursor.0] {
            if *name != " " && totalLine.len() + name.len() > self.cursor.1 {
                self.cursor.1 = totalLine.len() + name.len();
                return;
            }
            totalLine.push_str(name);
        }
    }
    
    pub fn MoveCursorLeft (&mut self, amount: usize) {
        if self.cursor.1 == 0 && self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.lines[self.cursor.0].len();
            return;
        }
        
        self.cursor = (
            self.cursor.0,
            std::cmp::min(
                self.cursor.1,
                self.lines[self.cursor.0].len()
            ).saturating_sub(amount)
        );
    }

    pub fn MoveCursorRight (&mut self, amount: usize) {
        if self.cursor.1 == self.lines[self.cursor.0].len() && self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
            return;
        }

        self.cursor = (
            self.cursor.0,
            self.cursor.1.saturating_add(amount)
        );
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

        (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);
    }

    pub fn UnIndent (&mut self) {
        // checking for 4 spaces at the start
        if let Some(charSet) = &self.lines[self.cursor.0].get(..4) {
            if *charSet == "    " {
                for _ in 0..4 {  self.lines[self.cursor.0].remove(0);  }
                self.cursor.1 = self.cursor.1.saturating_sub(4);

                self.RecalcTokens(self.cursor.0);

                (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);
            }
        }
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

    pub fn JumpCursor (&mut self, position: usize, scalar01: usize) {
        self.cursor.0 =
            std::cmp::min(
                position,
                self.lines.len() - 1
        );
        
        // finding the starting position
        let mut startingPos = self.lines[self.cursor.0].len() * scalar01;
        for i in 0..self.lines[self.cursor.0].len() {
            startingPos += 1;
            if self.lines[self.cursor.0].get(i..i+1).unwrap_or("") != " " {
                break;
            }
        }
        self.cursor.1 = std::cmp::min(
            startingPos,
            self.lines[self.cursor.0].len()
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

            (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);

            self.cursor.1 = 0;
            self.CursorDown();
            return;
        }

        let rightSide = self.lines[self.cursor.0]
            .split_off(std::cmp::min(
                self.cursor.1,
            length
        ));

        self.lines.insert(
            self.cursor.0 + 1,
            rightSide,
        );
        self.lineTokens.insert(
            self.cursor.0 + 1,
            vec!(),
        );
        
        self.RecalcTokens(self.cursor.0);
        self.RecalcTokens(self.cursor.0 + 1);
        self.cursor.1 = 0;
        self.CursorDown();
        
        (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);

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

            (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);

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

        (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);
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
                text.light_red()
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
                text.light_green()
            },
            TokenType::Lifetime => {
                text.light_blue()
            },
            TokenType::String => {
                text.yellow()
            },
            TokenType::Comment => {
                text.green()
            },
            TokenType::Null => {
                text.white()
            },
            TokenType::Primative => {
                text.light_yellow()
            },
            TokenType::Keyword => {
                text.light_red()
            }
        }
    } 

    pub fn GetScrolledText (&mut self, area: Rect, editingCode: bool) -> Vec <ratatui::text::Line> {
        // using the known area to adjust the scrolled position
        if self.scrolled + SCROLL_BOUNDS >= self.cursor.0 {
            if self.scrolled.saturating_sub(CENTER_BOUNDS) >= self.cursor.0 {
                let center = std::cmp::min(
                    self.cursor.0.saturating_sub((area.height as usize).saturating_sub(10) / 2),
                    self.lines.len() - 1
                );
                self.scrolled = center;
            } else {
                self.scrolled = self.cursor.0.saturating_sub(SCROLL_BOUNDS);
            }
        }
        if (self.scrolled + area.height as usize - 12).saturating_sub(SCROLL_BOUNDS) <= self.cursor.0 {
            if self.scrolled + area.height as usize + CENTER_BOUNDS <= self.cursor.0 {
                let center = std::cmp::min(
                    self.cursor.0.saturating_sub((area.height as usize).saturating_sub(10) / 2),
                    self.lines.len() - 1
                );
                self.scrolled = center;
            } else {
                self.scrolled = (self.cursor.0 + SCROLL_BOUNDS).saturating_sub(area.height as usize - 12);
            }
        }
        
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
            
            // adjust this for the total length of the file so everything is held to the same line length
            let totalSize = (self.lines.len()).to_string().len() + 1;  // number of digits + 2usize;
            for _ in 0..totalSize {
                if lineNumberText.len() <= totalSize {
                    lineNumberText.push(' ');
                }
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
            lineTokens: vec![],
            scopeJumps: vec![],
            scopes: ScopeNode {
                children: vec![],
                name: "Root".to_string(),
                start: 0,
                end: 0,
            },
            linearScopes: vec![],
            scrolled: 0,
            name: "Welcome.txt"
                .to_string(),
        }
    }
}


#[derive(Debug)]
pub struct CodeTabs {
    tabFileNames: Vec <String>,
    tabs: Vec <CodeTab>,
    currentTab: usize,
}

impl CodeTabs {
    pub fn GetScrolledText (&mut self, area: Rect, editingCode: bool) -> Vec <ratatui::text::Line> {
        self.tabs[self.currentTab].GetScrolledText(area, editingCode)
    }
}

impl CodeTabs {

    pub fn CloseTab (&mut self) {
        if self.tabs.len() > 1 {  // there needs to be at least one file open
            self.tabs.remove(self.currentTab);
            self.tabFileNames.remove(self.currentTab);
            self.currentTab = self.currentTab.saturating_sub(1);
        }
    }

    pub fn MoveTabRight (&mut self) {
        if self.currentTab < self.tabFileNames.len() - 1 {
            self.currentTab += 1;

            self.tabFileNames.swap(self.currentTab, self.currentTab - 1);
            self.tabs.swap(self.currentTab, self.currentTab - 1);
        }
    }

    pub fn MoveTabLeft (&mut self) {
        if self.currentTab > 0 {
            self.currentTab -= 1;  // there's a condition ensuring it's 1 or greater

            self.tabFileNames.swap(self.currentTab, self.currentTab + 1);
            self.tabs.swap(self.currentTab, self.currentTab + 1);
        }
    }

    pub fn TabLeft (&mut self) {
        self.currentTab = self.currentTab.saturating_sub(1);
    }

    pub fn TabRight(&mut self) {
        self.currentTab = std::cmp::min(
            self.currentTab.saturating_add(1),
            self.tabFileNames.len() - 1
        );
    }

    pub fn GetColoredNames (&self, onTabs: bool) -> Vec <Span> {
        let mut colored = vec!();

        if onTabs {
            for (index, tab) in self.tabFileNames.iter().enumerate() {
                if index == self.currentTab {
                    colored.push(
                        format!(" ({}) ", index + 1).to_string().light_yellow().bold().on_dark_gray().underlined()
                    );
                    colored.push(
                        tab.clone().white().italic().on_dark_gray().underlined()
                    );
                    colored.push(
                        " |".to_string().white().bold().on_dark_gray().underlined()
                    );
                    continue;
                }
                colored.push(
                    format!(" ({}) ", index + 1).to_string().light_yellow().bold().underlined()
                );
                colored.push(
                    tab.clone().white().italic().underlined()
                );
                colored.push(
                    " |".to_string().white().bold().underlined()
                );
            }
            return colored;
        }

        for (index, tab) in self.tabFileNames.iter().enumerate() {
            if index == self.currentTab {
                colored.push(
                    format!(" ({}) ", index + 1).to_string().light_yellow().bold().on_dark_gray()
                );
                colored.push(
                    tab.clone().white().italic().on_dark_gray()
                );
                colored.push(
                    " |".to_string().white().bold().on_dark_gray()
                );
                continue;
            }
            colored.push(
                format!(" ({}) ", index + 1).to_string().light_yellow().bold()
            );
            colored.push(
                tab.clone().white().italic()
            );
            colored.push(
                " |".to_string().white().bold()
            );
        }

        colored
    }
}

impl Default for CodeTabs {
    fn default() -> Self {
        let mut tabs = CodeTabs {
            tabFileNames: vec![
                "main.rs".to_string()
            ],
            tabs: vec![
                CodeTab {
                    cursor: (0, 0),
                    lines: {
                        // temporary
                        let mut lines: Vec <String> = vec!();
                        
                        let contents = std::fs::read_to_string("src/main.rs").unwrap();
                        let mut current = String::new();
                        for chr in contents.chars() {
                            if chr == '\n' {
                                lines.push(current.clone());
                                current.clear();
                            } else {
                                current.push(chr);
                            }
                        }

                        lines
                    },
                    lineTokens: vec![],
                    scopeJumps: vec![],
                    scopes: ScopeNode {
                        children: vec![
                            ScopeNode {
                                children: vec![],
                                name: "fn main".to_string(),
                                start: 0,
                                end: 2,
                            }
                        ],
                        name: "Root".to_string(),
                        start: 0,
                        end: 2,
                    },
                    linearScopes: vec![
                        vec![0]
                    ],
                    scrolled: 0,
                    name: "main.rs".to_string(),
                }

            ],  // put a tab here or something idk
            currentTab: 0
         };

         for tab in &mut tabs.tabs {
            tab.lineTokens.clear();
            for (index, line) in tab.lines.iter().enumerate() {
                tab.lineTokens.push(
                    GenerateTokens(line.clone())
                );
            }
            (tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens);
        }

        tabs
            /*tabFileNames: vec![
                "Welcome.txt".to_string(),
                "HelloWorld.rs".to_string()
            ],
            tabs: vec![
                CodeTab {
                    cursor: (0, 0),
                    lines: vec![
                        "Welcome! Please open or create a file...".to_string()
                    ],
                    lineTokens: vec![
                        GenerateTokens("Welcome! Please open or create a file...".to_string())
                    ],
                    scopeJumps: vec![
                        vec![]
                    ],
                    scopes: ScopeNode {
                        children: vec![],
                        name: "Root".to_string(),
                        start: 0,
                        end: 0,
                    },
                    linearScopes: vec![],
                    scrolled: 0,
                    name: "Welcome.txt".to_string(),
                },
                CodeTab {
                    cursor: (0, 0),
                    lines: vec![
                        "fn main () {".to_string(),
                        "    println!(\"Hello World!\");".to_string(),
                        "}".to_string(),
                    ],
                    lineTokens: vec![
                        GenerateTokens("fn main () {".to_string()),
                        GenerateTokens("    println!(\"Hello World!\");".to_string()),
                        GenerateTokens("}".to_string())
                    ],
                    scopeJumps: vec![
                        vec![0],
                        vec![0],
                        vec![0],
                    ],
                    scopes: ScopeNode {
                        children: vec![
                            ScopeNode {
                                children: vec![],
                                name: "fn main".to_string(),
                                start: 0,
                                end: 2,
                            }
                        ],
                        name: "Root".to_string(),
                        start: 0,
                        end: 2,
                    },
                    linearScopes: vec![
                        vec![0]
                    ],
                    scrolled: 0,
                    name: "HelloWorld.rs".to_string(),
                }

            ],  // put a tab here or something idk
            currentTab: 0
        }*/
    }
}



#[derive(Debug, Default)]
pub enum FileTabs {
    Outline,
    #[default] Files,
}

#[derive(Debug, Default)]
pub struct FileBrowser {
    files: Vec <String>,  // stores the names
    currentFile: usize,  // stores the index of the file and it's name

    fileTab: FileTabs,
    fileCursor: usize,
    outlineCursor: usize,
}

impl FileBrowser {
    pub fn MoveCursorDown (&mut self, outline: &Vec < Vec<usize>>, _rootNode: &ScopeNode) {
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

#[derive(Default)]
pub struct KeyParser {
    keyModifiers: Vec <KeyModifiers>,
    keyEvents: std::collections::HashMap <KeyCode, bool>,
    charEvents: Vec <char>,
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
        }
    }

    pub fn ClearEvents (&mut self) {
        self.charEvents.clear();
        self.keyModifiers.clear();
        self.keyEvents.clear();
    }

    pub fn ContainsChar (&self, chr: char) -> bool {
        self.charEvents.contains(&chr)
    }

    pub fn ContainsModifier (&self, modifider: KeyModifiers) -> bool {
        self.keyModifiers.contains(&modifider)
    }

    pub fn ContainsKeyCode (&self, key: KeyCode) -> bool {
        *self.keyEvents.get(&key).unwrap_or(&false)
    }

}

impl Perform for KeyParser {
    fn execute(&mut self, byte: u8) {
        match byte {
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
        if chr as u8 == 0x7F {
            self.keyEvents.insert(KeyCode::Delete, true);
            return;
        }
        //println!("char {}: '{}'", chr as u8, chr);
        self.charEvents.push(chr);
    }
    
    fn csi_dispatch(&mut self, params: &vte::Params, _: &[u8], _: bool, c: char) {
        let numbers: Vec<u16> = params.iter().map(|p| p[0]).collect();
        //for number in &numbers {println!("{}", number);}
        if c == '~' {  // this section is for custom escape codes
            if numbers == [3, 2] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Shift);
            } else if numbers == [3, 3] {
                self.keyEvents.insert(KeyCode::Delete, true);
                self.keyModifiers.push(KeyModifiers::Option);
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
                    }
                },
                0x43 => {
                    self.keyEvents.insert(KeyCode::Right, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    }
                },
                0x41 => {
                    self.keyEvents.insert(KeyCode::Up, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
                    }
                },
                0x42 => {
                    self.keyEvents.insert(KeyCode::Down, true);
                    if numbers == [1, 3] {
                        self.keyModifiers.push(KeyModifiers::Option);
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
    codeTabs: CodeTabs,
    currentCommand: String,
    fileBrowser: FileBrowser,
}

impl App {

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        enable_raw_mode()?; // Enable raw mode for direct input handling

        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        
        let mut parser = Parser::new();
        let mut keyParser = KeyParser::new();
        let mut buffer = [0; 10];
        let mut stdin = tokio::io::stdin();
        
        while !self.exit {
            if self.exit {
                break;
            }

            buffer.fill(0);
            
            tokio::select! {
                result = stdin.read(&mut buffer) => {
                    if let Ok(n) = result {
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
                _ = tokio::time::sleep(std::time::Duration::from_millis(16)) => {
                    terminal.draw(|frame| self.draw(frame))?;
                    if self.exit {
                        break;
                    }
                },
            }

            self.HandleKeyEvents(&keyParser);
            keyParser.ClearEvents();
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn HandleKeyEvents (&mut self, keyEvents: &KeyParser) {

        match self.appState {
            AppState::CommandPrompt => {
                for chr in &keyEvents.charEvents {
                    self.currentCommand.push(*chr);
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

                                    self.currentCommand.clear();
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

                                    self.currentCommand.clear();
                                }
                            }
                        }

                        self.currentCommand.clear();
                    } else if keyEvents.ContainsKeyCode(KeyCode::Delete) {
                        self.currentCommand.pop();
                    }
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
                        if keyEvents.ContainsKeyCode(KeyCode::Return) {
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
                        }
                    },
                }
            },
            AppState::Tabs => {
                match self.tabState {
                    TabState::Code => {
                        for chr in &keyEvents.charEvents {
                            self.codeTabs.tabs[self.codeTabs.currentTab]
                                .InsertChars(chr.to_string());
                        }

                        if keyEvents.ContainsKeyCode(KeyCode::Delete) {
                            let mut numDel = 1;

                            if keyEvents.keyModifiers.contains(&KeyModifiers::Option) {
                                numDel = self.codeTabs.tabs[self.codeTabs.currentTab].FindTokenPosLeft();
                            }

                            self.codeTabs.tabs[
                                self.codeTabs.currentTab
                            ].DelChars(numDel, 0);
                        } else if keyEvents.ContainsKeyCode(KeyCode::Left) {
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorLeftToken();
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorLeft(1);
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Right) {
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorRightToken();
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorRight(1);
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Up) {
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                                let mut jumps = tab.scopeJumps[tab.cursor.0].clone();
                                jumps.reverse();
                                tab.JumpCursor( 
                                    tab.scopes.GetNode(&mut jumps).start, 1
                                );
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].CursorUp();
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Down) {
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                                let mut jumps = tab.scopeJumps[tab.cursor.0].clone();
                                jumps.reverse();
                                tab.JumpCursor( tab.scopes.GetNode(&mut jumps).end, 1);
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].CursorDown();
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Tab) {
                            if keyEvents.ContainsModifier(KeyModifiers::Shift) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].UnIndent();
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab]
                                    .InsertChars("    ".to_string());
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Return) {
                            self.codeTabs.tabs[self.codeTabs.currentTab].LineBreakIn();
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

    /*
    fn HandleEvents(&mut self) -> io::Result<()> {
        let event = event::read()?;

        /*if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Esc => {
                    self.escapeSeq.clear();
                    self.escapeSeq.push('\x1B'); // Start buffering
                }
                KeyCode::Char(c) => {
                    self.escapeSeq.push(c);
                    if self.escapeSeq == "\x1B[3;2~" {
                        println!("Shift+Delete detected!");
                        self.escapeSeq.clear();
                    } else if !self.escapeSeq.starts_with("\x1B[") {
                        self.escapeSeq.clear(); // Reset if it doesn't match
                    }
                }
                _ => {
                    self.escapeSeq.clear(); // Reset buffer for unrelated keys
                }
            }
        }*/

        match event {//event::read()? {
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
        /*
        match key_event.code {
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

        /*
        //println!("Key Event: {:?}", key_event); // Debugging output
        match key_event.code {
            KeyCode::Esc => {
                //println!("Escape key detected! Waiting for additional input...");
                self.escapeSeq.clear();
                self.escapeSeq.push('\x1B'); // Store the escape prefix
            },
            KeyCode::Char(c) => {
                //println!("Character detected: '{}'", c);
                if !self.escapeSeq.is_empty() {
                    self.escapeSeq.push(c);

                    //println!("Esc Seq: {}", self.escapeSeq);
                    if self.escapeSeq.trim() == "\x1B[3;2~" {
                        println!("Shift+Delete detected!");
                        self.escapeSeq.clear(); // Reset buffer
                    }
                }
            },
            _ => {
                self.escapeSeq.clear();
            },
        };
        //println!("Current escape sequence: {:?}", self.escapeSeq);// */

        match key_event.code {
            KeyCode::Enter if matches!(self.appState, AppState::CommandPrompt) && self.currentCommand.len() > 0 => {
                if self.currentCommand == "q" {
                    self.Exit();
                }

                if self.currentCommand.chars().next().unwrap() == '[' {
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
                }
                if self.currentCommand.chars().next().unwrap() == ']' {
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

                self.currentCommand = "".to_string();
            }
            KeyCode::Left if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorLeftToken();
                } else {
                    self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorLeft(1);
                }
            },
            KeyCode::Right if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorRightToken();
                } else {
                    self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorRight(1);
                }
            },
            
            KeyCode::Left if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Tabs) => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.codeTabs.MoveTabLeft()
                } else {
                    self.codeTabs.TabLeft();
                }
            },
            KeyCode::Right if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Tabs) => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.codeTabs.MoveTabRight()
                } else {
                    self.codeTabs.TabRight();
                }
            },
            
            KeyCode::Down if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Files) =>
                self.fileBrowser.MoveCursorDown(
                        &self.codeTabs.tabs[self.codeTabs.currentTab].linearScopes,
                        &self.codeTabs.tabs[self.codeTabs.currentTab].scopes),
            KeyCode::Up if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Files) =>
                self.fileBrowser.MoveCursorUp(),

            KeyCode::Backspace => {
                if matches!(self.appState, AppState::CommandPrompt) {
                    if self.currentCommand.len() > 0 {
                        self.currentCommand.pop();
                    } else if matches!(self.tabState, TabState::Tabs) {
                        self.codeTabs.CloseTab();
                    }
                } else if matches!(self.tabState, TabState::Code) {
                    let numDel = 1;

                    /*println!("Key: {}", key_event.modifiers);
                    if key_event.modifiers.contains(KeyModifiers::ALT) {
                        numDel = self.codeTabs.tabs[self.codeTabs.currentTab].FindTokenPosLeft();
                    }*/  // not working for some reason; option isn't sent when using delete :(

                    self.codeTabs.tabs[
                        self.codeTabs.currentTab
                    ].DelChars(numDel, 0);
                }
            },
            KeyCode::Esc => {
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
            },
            KeyCode::Enter if matches!(self.appState, AppState::CommandPrompt) && !matches!(self.tabState, TabState::Files) => {
                self.appState = AppState::Tabs;
                self.tabState = TabState::Code;
            },
            KeyCode::BackTab => {
                if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) {
                    self.codeTabs.tabs[self.codeTabs.currentTab].UnIndent();
                } else if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Files) {
                    self.fileBrowser.fileTab = match self.fileBrowser.fileTab {
                        FileTabs::Files => FileTabs::Outline,
                        FileTabs::Outline => FileTabs::Files,
                    }
                }
            },
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
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                    let mut jumps = tab.scopeJumps[tab.cursor.0].clone();
                    jumps.reverse();
                    tab.JumpCursor( tab.scopes.GetNode(&mut jumps).end, 1);
                } else {
                    self.codeTabs.tabs[self.codeTabs.currentTab].CursorDown();
                }
            },
            KeyCode::Up if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                    let mut jumps = tab.scopeJumps[tab.cursor.0].clone();
                    jumps.reverse();
                    tab.JumpCursor( 
                        tab.scopes.GetNode(&mut jumps).start, 1
                    );
                } else {
                    self.codeTabs.tabs[self.codeTabs.currentTab].CursorUp();
                }
            },
            KeyCode::Enter if matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {
                /*if key_event.modifiers.contains(KeyModifiers::SUPER) {  // command; alt == option; super == command
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        self.codeTabs.tabs[self.codeTabs.currentTab].LineBreakBefore();
                    } else {
                        self.codeTabs.tabs[self.codeTabs.currentTab].LineBreakAfter();
                    }
                } else */{
                    self.codeTabs.tabs[self.codeTabs.currentTab].LineBreakIn();
                }
            },
            KeyCode::Enter if matches!(self.appState, AppState::CommandPrompt) && matches!(self.tabState, TabState::Files) && matches!(self.fileBrowser.fileTab, FileTabs::Outline) => {
                // getting the line number the cursor is one
                let mut nodePath = self.codeTabs.tabs[self.codeTabs.currentTab].linearScopes[
                    self.fileBrowser.outlineCursor].clone();
                nodePath.reverse();
                let node = self.codeTabs.tabs[self.codeTabs.currentTab].scopes.GetNode(
                        &mut nodePath
                );
                let start = node.start;
                self.codeTabs.tabs[self.codeTabs.currentTab].JumpCursor(start, 1);
            },
            // these are good, i just removed escapeSeq and don't want errors. Keep these when converting to the new handler
            /*KeyCode::Char(to_insert) if self.escapeSeq.is_empty() && matches!(self.appState, AppState::Tabs) && matches!(self.tabState, TabState::Code) => {// && !validSeq => {
                self.codeTabs.tabs[self.codeTabs.currentTab]
                    .InsertChars(to_insert.to_string());
            }
            KeyCode::Char(to_insert) if self.escapeSeq.is_empty() && matches!(self.appState, AppState::CommandPrompt) => {// && !validSeq => {
                self.currentCommand.push(to_insert);
            }*/
            _ => {}
        }

        //if validSeq {  self.escapeSeq.clear();  }
    }
    */

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
                width: area.width - 20,
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
                    if scopeIndex.len() == 0 {  continue;  }
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
                                        1 => scope.name.clone().light_blue().underlined(),
                                        2 => scope.name.clone().light_magenta().underlined(),
                                        3 => scope.name.clone().light_red().underlined(),
                                        4 => scope.name.clone().light_yellow().underlined(),
                                        5 => scope.name.clone().light_green().underlined(),
                                        _ => scope.name.clone().white().underlined(),
                                    }
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
            fileText = Text::from(fileStringText);
        } else {
            fileText = Text::from(vec![
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

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal).await;
    ratatui::restore();
    app_result
}

