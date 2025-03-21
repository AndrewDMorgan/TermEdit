// snake case is just bad
#![allow(non_snake_case)]

use std::default;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use vte::{Parser, Perform};

use crossterm::terminal::enable_raw_mode;

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
        "?".to_string(),
        "=".to_string(),
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
        let prevToken = tokenStrs.get(index.saturating_sub(1)).unwrap_or(emptyString);
        
        let newFlag =
            match token.as_str() {
                "/" if nextToken == "/" || nextToken == "*" => TokenFlags::Comment,
                "*" if nextToken == "/" => TokenFlags::Null,
                "\"" if !matches!(currentFlag, TokenFlags::Comment | TokenFlags::Char) && prevToken != "\\" => {
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
                    "use" | "mod" | "None" | "Some" | "Ok" | "Err" |
                    "async" | "await" | "default" | "derive" | "new" |
                    "as" | "?" => TokenType::Keyword,
                " " => TokenType::Null,
                "i32" | "isize" | "i16" | "i8" | "i128" | "i64" |
                    "u32" | "usize" | "u16" | "u8" | "u128" | "u64" | 
                    "f16" | "f32" | "f64" | "f128" | "String" |
                    "str" | "Vec" | "bool" | "char" | "Result" |
                    "Option" => TokenType::Primative,
                "[" | "]" => TokenType::Bracket,
                "(" | ")" => TokenType::Parentheses,
                "#" => TokenType::Macro,
                _s if nextToken == "!" => TokenType::Macro,
                ":" => TokenType::Member,
                //"" => TokenType::Variable,
                s if s.chars().next().map_or(false, |c| {
                    c.is_ascii_digit()
                }) => TokenType::Number,
                "=" | "-" if nextToken == ">" => TokenType::Keyword,
                ">" if prevToken == "=" => TokenType::Keyword,
                ">" if prevToken == "-" => TokenType::Keyword,
                "=" if prevToken == ">" || prevToken == "<" || prevToken == "=" => TokenType::Logic,
                s if (prevToken == "&" && s == "&") || (prevToken == "|" && s == "|") => TokenType::Logic,
                s if (nextToken == "&" && s == "&") || (nextToken == "|" && s == "|") => TokenType::Logic,
                ">" | "<" | "false" | "true" | "!" => TokenType::Logic,
                "=" if nextToken == "=" => TokenType::Logic,
                "=" if prevToken == "+" || prevToken == "-" || prevToken == "*" || prevToken == "/" => TokenType::Math,
                "=" if nextToken == "+" || nextToken == "-" || nextToken == "*" || nextToken == "/" => TokenType::Math,
                "+" | "-" | "*" | "/" => TokenType::Math,
                "{" | "}" | "|" => TokenType::SquirlyBracket,
                "let" | "=" | "mut" => TokenType::Assignment,
                ";" => TokenType::Endl,
                "&" => TokenType::Barrow,
                "'" if matches!(flags[index], TokenFlags::Generic) => TokenType::Lifetime,
                _s if matches!(flags[index], TokenFlags::Generic) && prevToken == "'" => TokenType::Lifetime,
                "a" | "b" if prevToken == "'" && (nextToken == "," || nextToken == ">" || nextToken == " ") => TokenType::Lifetime,
                "\"" | "'" => TokenType::String,
                "enum" | "pub" | "struct" | "impl" | "self" | "Self" => TokenType::Object,
                _s if strToken.to_uppercase() == *strToken => TokenType::Const,
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
            let (_lastToken, lastName) = {
                if index == 0 {  &(TokenType::Null, "".to_string())  }
                else {  &tokens[index - 1]  }
            };
            if !(matches!(token, TokenType::Comment) || (lastName == "\"" || lastName == "'") && matches!(token, TokenType::String)) {
                // checking bracket depth
                if name == "{" {
                    bracketDepth += 1;
                } else if name == "}" {
                    bracketDepth -= 1;
                }
            }

            // checking for something to define the name
            if matches!(token, TokenType::Keyword | TokenType::Object | TokenType::Function) {
                if !(VALID_NAMES_NEXT.contains(&name.trim()) || VALID_NAMES_TAKE.contains(&name.trim())) {
                    continue;
                }
                
                // checking the scope to see if ti continues or ends on the same line
                let mut brackDepth = 0isize;
                for (indx, (token, name)) in tokens.iter().enumerate() {
                    let (_lastToken, lastName) = {
                        if indx == 0 {  &(TokenType::Null, "".to_string())  }
                        else {  &tokens[indx - 1]  }
                    };
                    if !(matches!(token, TokenType::Comment) || lastName == "\"" && matches!(token, TokenType::String)) &&
                        indx > index 
                    {
                        if name == "{" {
                            brackDepth += 1;
                        } else if name == "}" {
                            brackDepth -= 1;
                        }
                    }
                }

                // check bracketdepth here so any overlapping marks are accounted for
                if bracketDepth < 0 && brackDepth > 0 {  // not pushing any jumps bc/ it'll be done later
                    let mut scopeCopy = currentScope.clone();
                    scopeCopy.reverse();
                    rootNode.SetEnd(&mut scopeCopy, lineNumber);
                    currentScope.pop();
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
    mouseScrolled: isize,
    mouseScrolledFlt: f64,
    name: String,
    fileName: String
}

impl CodeTab {

    pub fn Save (&self) {
        let mut fileContents = String::new();
        for line in &self.lines {
            fileContents.push_str(line.as_str());
            fileContents.push('\n');
        }
        fileContents.pop();  // popping the final \n so it doesn't gradually expand over time
        
        std::fs::write(&self.fileName, fileContents).expect("Unable to write file");
    }

    pub fn MoveCursorLeftToken (&mut self) {
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
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

    pub fn FindTokenPosRight (&mut self) -> usize {
        if self.lines[self.cursor.0].is_empty() {  return 0;  }

        self.cursor.1 = std::cmp::min (
            self.cursor.1,
            self.lines[self.cursor.0].len()
        );
        let mut newCursor = self.cursor.1;

        while newCursor < self.lines[self.cursor.0].len()-1 &&
            self.lines[self.cursor.0].get(newCursor..newCursor + 1).unwrap_or("") == " "
        {
            
            newCursor += 1;
        }

        let mut totalLine = String::new();
        for (_token, name) in &self.lineTokens[self.cursor.0] {
            if totalLine.len() + name.len() > newCursor {
                newCursor = totalLine.len() + name.len();
                break;
            }
            totalLine.push_str(name);
        }

        newCursor - self.cursor.1
    }
    
    pub fn MoveCursorRightToken (&mut self) {
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
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
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        if (self.cursor.1 == 0 || self.lines[self.cursor.0].is_empty()) && self.cursor.0 > 0 {
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
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        if self.cursor.1 >= self.lines[self.cursor.0].len() && self.cursor.0 < self.lines.len() - 1 {
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
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
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
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
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
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        self.cursor = (
            self.cursor.0.saturating_sub(1),
            self.cursor.1
        );
    }

    pub fn CursorDown (&mut self) {
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        self.cursor = (
            std::cmp::min(
                self.cursor.0.saturating_add(1),
                self.lines.len() - 1
            ),
            self.cursor.1
        );
    }

    pub fn JumpCursor (&mut self, position: usize, scalar01: usize) {
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
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

    pub fn LineBreakIn (&mut self) {
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
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

    // cursorOffset can be used to delete in multiple directions
    // if the cursorOffset is equal to numDel, it'll delete to the right
    // cursorOffset = 0 is default and dels to the left
    pub fn DelChars (&mut self, numDel: usize, cursorOffset: usize) {
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
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

        let mut newCursor = self.cursor.1;
        if cursorOffset == 0 {
            newCursor = self.cursor.1.saturating_sub(numDel);
        }

        self.lines[self.cursor.0]
            .replace_range(
                newCursor
                    ..
                    std::cmp::min(
                        self.cursor.1.saturating_add(cursorOffset),
                        length
                ),
                ""
        );

        self.cursor.1 = newCursor;

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
                text.light_red().bold()
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
                text.blue().italic()
            },
            TokenType::Const => {
                text.cyan().italic()
            },
            TokenType::Barrow => {
                text.light_green().italic()
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
                text.light_red().bold()
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

        let scroll = std::cmp::max(self.scrolled as isize + self.mouseScrolled, 0) as usize;
        
        let mut tabText = vec![];
        
        for lineNumber in scroll..(scroll + area.height as usize - 10) {
            if lineNumber >= self.lines.len() {  continue;  }

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

            let mut coloredLeft: Vec <(usize, Span)> = vec!();
            let mut coloredRight: Vec <(usize, Span)> = vec!();

            if lineNumber == self.cursor.0 {
                coloredLeft.push((lineNumberText.len(), lineNumberText.red().bold().add_modifier(Modifier::UNDERLINED)));
            } else {
                coloredLeft.push((lineNumberText.len(), lineNumberText.gray().italic()));
            }

            let mut currentCharNum = 0;
            for (token, text) in &self.lineTokens[lineNumber] {
                if lineNumber == self.cursor.0 && currentCharNum + text.len() > self.cursor.1 {
                    if currentCharNum >= self.cursor.1 {
                        if currentCharNum == self.cursor.1 && editingCode {
                            coloredLeft.push((1, "|".to_string().white().bold()));
                        }
                        coloredRight.push((text.len(), self.GenerateColor(token, text.as_str())));
                    } else {
                        let txt = &text[0..text.len() - (
                            currentCharNum + text.len() - self.cursor.1
                        )];
                        coloredLeft.push((
                            txt.len(),
                            self.GenerateColor(token, txt)
                        ));
                        if editingCode {  coloredLeft.push((1, "|".to_string().white().bold()))  };
                        let txt = &text[
                            text.len() - (
                                currentCharNum + text.len() - self.cursor.1
                            )..text.len()
                        ];
                        coloredRight.push((
                            txt.len(),
                            self.GenerateColor(token, txt)
                        ));
                    }
                } else {
                    coloredLeft.push((text.len(), self.GenerateColor(token, text.as_str())));
                }

                currentCharNum += text.len();
            }
            if lineNumber == self.cursor.0 && currentCharNum <= self.cursor.1 && editingCode {
                coloredLeft.push((1, "|".to_string().white().bold()));
            }

            let mut charCount = 0usize;
            let mut finalColText: Vec <Span> = vec!();
            for (size, col) in coloredLeft {
                if charCount + size >= (area.width - 29) as usize {  break;  }
                finalColText.push(col);
                charCount += size;
            } for (size, col) in coloredRight {
                if charCount + size >= (area.width - 29) as usize {  break;  }
                finalColText.push(col);
                charCount += size;
            }

            if self.cursor.0 == lineNumber && editingCode{
                tabText.push(Line::from(
                    finalColText
                ).add_modifier(Modifier::UNDERLINED));
            } else {
                tabText.push(Line::from(
                    finalColText
                ));
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
            mouseScrolled: 0,
            mouseScrolledFlt: 0.0,
            name: "Welcome.txt"
                .to_string(),
            fileName: "".to_string(),
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
        CodeTabs {
            tabFileNames: vec![],
            tabs: vec![
                CodeTab {
                    cursor: (0, 0),
                    lines: vec!(),
                    lineTokens: vec![],
                    scopeJumps: vec![],
                    scopes: ScopeNode {
                        children: vec![],
                        name: "Root".to_string(),
                        start: 0,
                        end: 2,
                    },
                    linearScopes: vec![
                        vec![0]
                    ],
                    scrolled: 0,
                    mouseScrolled: 0,
                    mouseScrolledFlt: 0.0,
                    name: "main.rs".to_string(),
                    fileName: "".to_string(),
                }

            ],  // put a tab here or something idk
            currentTab: 0
        }
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

    fileTab: FileTabs,
    fileCursor: usize,
    outlineCursor: usize,
}

impl FileBrowser {
    pub fn LoadFilePath (&mut self, pathInput: &str, codeTabs: &mut CodeTabs) {
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
                            GenerateTokens(line.clone())
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
        }
    }

    pub fn ClearEvents (&mut self) {
        self.charEvents.clear();
        self.keyModifiers.clear();
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

    pub fn ContainsModifier (&self, modifider: KeyModifiers) -> bool {
        self.keyModifiers.contains(&modifider)
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

                // adding key press modifiers
                if (byte & 32) != 0 {
                    self.keyModifiers.push(KeyModifiers::Shift);
                } if (byte & 64) != 0 {
                    self.keyModifiers.push(KeyModifiers::Option);
                } if (byte & 128) != 0 {
                    self.keyModifiers.push(KeyModifiers::Control);
                }

                let is_scroll = (byte & 64) != 0;
                let eventType = match (is_scroll, button) {
                    (true, 0) => MouseEventType::Up,   // 1???? ig so
                    (true, 1) => MouseEventType::Down, // 2???? ig so
                    (false, 0) => MouseEventType::Left,
                    (false, 1) => MouseEventType::Middle,
                    (false, 2) => MouseEventType::Right,
                    _ => MouseEventType::Null,
                };
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
            self.HandleKeyEvents(&keyParser);
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
                    if matches!(event.state, MouseState::Release) {
                        if event.position.0 > 29 && event.position.1 < 10 + self.area.height && event.position.1 > 2 {
                            let tab = &mut self.codeTabs.tabs[self.codeTabs.currentTab];
                            let linePos = (std::cmp::max(tab.scrolled as isize + tab.mouseScrolled, 0) as usize +
                                event.position.1.saturating_sub(4) as usize,
                                event.position.0.saturating_sub(37) as usize);
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
                            tab.mouseScrolled = 0;
                            tab.mouseScrolledFlt = 0.0;
                        } else {
                            // todo!
                        }
                    }
                },
                MouseEventType::Middle => {},
                MouseEventType::Right => {},
                _ => {},
            }
        }
    }

    fn HandleKeyEvents (&mut self, keyEvents: &KeyParser) {

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
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorLeft(1);
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Right) {
                            if keyEvents.ContainsModifier(KeyModifiers::Option) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].MoveCursorRightToken();
                            } else if keyEvents.ContainsModifier(KeyModifiers::Command) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt = 0.0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled = 0;

                                let cursorLine = self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].cursor.1 =
                                    self.codeTabs.tabs[self.codeTabs.currentTab].lines[cursorLine].len();
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
                            } else if keyEvents.ContainsModifier(KeyModifiers::Command) {
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolledFlt = 0.0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].mouseScrolled = 0;
                                self.codeTabs.tabs[self.codeTabs.currentTab].cursor.0 = 0;
                            } else {
                                self.codeTabs.tabs[self.codeTabs.currentTab].CursorUp();
                            }
                        } else if keyEvents.ContainsKeyCode(KeyCode::Down) {
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
                        } else if keyEvents.ContainsModifier(KeyModifiers::Command) &&
                            keyEvents.ContainsChar('s') {
                            
                            // saving the program
                            self.codeTabs.tabs[self.codeTabs.currentTab].Save();
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
                format!("Debug: {}", self.debugInfo).red().bold()
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
            <shift> + any line movement/jump -> highlight all text selected (including jumps from line # inputs)
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

