
use ratatui::{
    layout::Rect,
    style::{Stylize, Modifier},
    text::{Line, Span},
};

use crate::Tokens::*;


// the bounds from the screen edge at which the cursor will begin scrolling
const SCROLL_BOUNDS: usize = 12;
const CENTER_BOUNDS: usize = 0;


#[derive(Debug)]
pub struct CodeTab {
    pub cursor: (usize, usize),  // line pos, char pos inside line
    pub lines: Vec <String>,
    pub lineTokens: Vec <Vec <(TokenType, String)>>,
    pub scopeJumps: Vec <Vec <usize>>,  // points to the index of the scope (needs adjusting as the tree is modified)
    pub scopes: ScopeNode,
    pub linearScopes: Vec <Vec <usize>>,
    pub scrolled: usize,
    pub mouseScrolled: isize,
    pub mouseScrolledFlt: f64,
    pub name: String,
    pub fileName: String,
    pub cursorEnd: (usize, usize),  // for text highlighting
    pub highlighting: bool,
    pub pauseScroll: u128,
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
    
    pub fn MoveCursorLeft (&mut self, amount: usize, highlight: bool) {
        if self.highlighting && !highlight && self.cursor.0 > self.cursorEnd.0 ||
            self.cursor.1 > self.cursorEnd.1 && !highlight
        {
            (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
            self.highlighting = false;
            return;
        } else if self.highlighting && !highlight {
            self.highlighting = false;
            return;
        }
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

    pub fn MoveCursorRight (&mut self, amount: usize, highlight: bool) {
        if self.highlighting && !highlight && self.cursor.0 < self.cursorEnd.0 ||
            self.cursor.1 < self.cursorEnd.1 && self.cursor.0 == self.cursorEnd.1 && !highlight
        {
            (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
            self.highlighting = false;
            return;
        } else if self.highlighting && !highlight {
            self.highlighting = false;
            return;
        }
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
        self.HandleHighlight();  // doesn't need to exit bc/ chars should still be added

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

    pub fn CursorUp (&mut self, highlight: bool) {
        if self.highlighting && !highlight && self.cursor.0 > self.cursorEnd.0 ||
            self.cursor.1 > self.cursorEnd.1 && !highlight
        {
            (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
            self.highlighting = false;
        } else if self.highlighting && !highlight {
            self.highlighting = false;
        }
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        self.cursor = (
            self.cursor.0.saturating_sub(1),
            self.cursor.1
        );
    }

    pub fn CursorDown (&mut self, highlight: bool) {
        if self.highlighting && !highlight && self.cursor.0 < self.cursorEnd.0 ||
            self.cursor.1 < self.cursorEnd.1 && self.cursor.0 == self.cursorEnd.1 && !highlight
        {
            (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
            self.highlighting = false;
        } else if self.highlighting && !highlight {
            self.highlighting = false;
        }

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

    pub fn LineBreakIn (&mut self, highlight: bool) {
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        let length = self.lines[self.cursor.0].len();

        if length == 0 {
            self.lines.insert(self.cursor.0, "".to_string());
            self.lineTokens[self.cursor.0].clear();
            self.lineTokens.insert(self.cursor.0, vec!());

            (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);

            self.cursor.1 = 0;
            self.CursorDown(highlight);
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
        self.CursorDown(highlight);
        
        (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);

    }

    pub fn HandleHighlight (&mut self) -> bool {
        if self.highlighting && self.cursorEnd != self.cursor {
            if self.cursorEnd.0 < self.cursor.0 ||
                 self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 < self.cursor.1
            {
                if self.cursorEnd.0 == self.cursor.0 {
                    self.lines[self.cursorEnd.0].replace_range(self.cursorEnd.1..self.cursor.1, "");
                    self.RecalcTokens(self.cursor.0);
                } else {
                    self.lines[self.cursorEnd.0].replace_range(self.cursorEnd.1.., "");
                    self.RecalcTokens(self.cursorEnd.0);
                    self.lines[self.cursor.0].replace_range(..self.cursor.1, "");
                    self.RecalcTokens(self.cursor.0);
                    // go through any inbetween lines and delete them. Also delete one extra line so there aren't to blanks?
                    let numBetween = self.cursor.0 - self.cursorEnd.0 - 1;
                    for _ in 0..numBetween {
                        self.lines.remove(self.cursorEnd.0 + 1);
                        self.lineTokens.remove(self.cursorEnd.0 + 1);
                    }
                    // push the next line onto the first...
                    let nextLine = self.lines[self.cursorEnd.0 + 1].clone();
                    self.lines[self.cursorEnd.0].push_str(nextLine.as_str());
                    self.RecalcTokens(self.cursorEnd.0);
                    self.lines.remove(self.cursorEnd.0 + 1);
                    self.lineTokens.remove(self.cursorEnd.0 + 1);
                }
                
                self.highlighting = false;
                self.cursor = self.cursorEnd;
                (self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens);
                return true;
            } else {
                // swapping the cursor and ending points so the other calculations work
                (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
                return self.HandleHighlight();
            }
        } false
    }

    pub fn GetSelection (&self) -> String {
        let mut occumulation = String::new();

        if self.highlighting && self.cursor != self.cursorEnd {
            if self.cursorEnd.0 == self.cursor.0 {
                if self.cursorEnd.1 < self.cursor.1 {  // cursor on the smae line
                    let selection = &self.lines[self.cursor.0][self.cursorEnd.1..self.cursor.1];
                    occumulation.push_str(selection);
                } else {
                    let selection = &self.lines[self.cursor.0][self.cursor.1..self.cursorEnd.1];
                    occumulation.push_str(selection);
                }
            } else if self.cursor.0 > self.cursorEnd.0 {  // cursor highlighting downwards
                let selection = &self.lines[self.cursorEnd.0][self.cursorEnd.1..];
                occumulation.push_str(selection);
                occumulation.push('\n');

                // getting the center section
                let numBetween = self.cursor.0 - self.cursorEnd.0 - 1;
                for i in 0..numBetween {
                    let selection = &self.lines[self.cursorEnd.0 + 1 + i];
                    occumulation.push_str(selection.clone().as_str());
                    occumulation.push('\n');
                }

                let selection = &self.lines[self.cursor.0][..self.cursor.1];
                occumulation.push_str(selection);
            } else {  // cursor highlighting upwards
                let selection = &self.lines[self.cursor.0][self.cursor.1..];
                occumulation.push_str(selection);
                occumulation.push('\n');

                // getting the center section
                let numBetween = self.cursorEnd.0 - self.cursor.0 - 1;
                for i in 0..numBetween {
                    let selection = &self.lines[self.cursor.0 + 1 + i];
                    occumulation.push_str(selection.clone().as_str());
                    occumulation.push('\n');
                }

                let selection = &self.lines[self.cursorEnd.0][..self.cursorEnd.1];
                occumulation.push_str(selection);
            }
        } else {
            occumulation.push_str(self.lines[self.cursor.0].clone().as_str());
            occumulation.push('\n');  // fix this so that it always forces it to be pushed to a new line before
        }

        occumulation
    }

    // cursorOffset can be used to delete in multiple directions
    // if the cursorOffset is equal to numDel, it'll delete to the right
    // cursorOffset = 0 is default and dels to the left
    pub fn DelChars (&mut self, numDel: usize, cursorOffset: usize) {
        // deleting characters from scrolling
        if self.HandleHighlight() {  return;  }

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

        let ending = self.fileName.split('.').last().unwrap_or("");
        let newTokens = GenerateTokens(self.lines[lineNumber].clone(), ending);
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
                text.light_green().italic().bold()
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
                if text == "todo" || text == "!" || text == "error" || text == "condition" || text == "conditions" {  text.green()  }  // basic but it kinda does stuff idk
                else {  text.green().dim()  }
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
        // using the known area to adjust the scrolled position (even though this can now be done elsewise..... too lazy to move it)
        let currentTime = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Time went backwards...")
            .as_millis();
        if currentTime.saturating_sub(self.pauseScroll) > 125 {
            if self.scrolled + SCROLL_BOUNDS >= self.cursor.0 {
                if self.scrolled.saturating_sub(CENTER_BOUNDS) >= self.cursor.0 && !self.highlighting {
                    let center = std::cmp::min(
                        self.cursor.0.saturating_sub((area.height as usize).saturating_sub(10) / 2),
                        self.lines.len() - 1
                    );
                    self.scrolled = center;
                } else {
                    self.scrolled = self.cursor.0.saturating_sub(SCROLL_BOUNDS);
                    if self.highlighting {  // making sure the highlighting doesn't scroll at light speed
                        std::thread::sleep(std::time::Duration::from_millis(75));
                    }
                }
            }
            if (self.scrolled + area.height as usize - 12).saturating_sub(SCROLL_BOUNDS) <= self.cursor.0 {
                if self.scrolled + area.height as usize + CENTER_BOUNDS <= self.cursor.0 && !self.highlighting {
                    let center = std::cmp::min(
                        self.cursor.0.saturating_sub((area.height as usize).saturating_sub(10) / 2),
                        self.lines.len() - 1
                    );
                    self.scrolled = center;
                } else {
                    self.scrolled = (self.cursor.0 + SCROLL_BOUNDS).saturating_sub(area.height as usize - 12);
                    if self.highlighting {  // making sure the highlighting doesn't scroll at light speed
                        std::thread::sleep(std::time::Duration::from_millis(75));
                    }
                }
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
                        if self.highlighting && (self.cursorEnd.0 > self.cursor.0 || self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 > self.cursor.1) {
                            if self.highlighting && (lineNumber == self.cursorEnd.0 && currentCharNum+text.len() <= self.cursorEnd.1 ||
                                lineNumber == self.cursor.0 && currentCharNum >= self.cursor.1) && self.cursor.0 != self.cursorEnd.0 ||
                                (lineNumber > self.cursor.0 && lineNumber < self.cursorEnd.0) ||
                                (lineNumber == self.cursorEnd.0 && lineNumber == self.cursor.0 &&
                                currentCharNum >= self.cursor.1 && currentCharNum + text.len() <= self.cursorEnd.1)
                            {
                                coloredRight.push((text.len(), self.GenerateColor(token, text.as_str()).on_dark_gray()));
                            } else if self.highlighting && currentCharNum+text.len() > self.cursorEnd.1 && currentCharNum < self.cursorEnd.1 && lineNumber == self.cursorEnd.0 {   // can't be equal to cursor line
                                let txtRight = &text[self.cursorEnd.1 - currentCharNum..];
                                let txtLeft = &text[..self.cursorEnd.1 - currentCharNum];
                                coloredRight.push((text.len(), self.GenerateColor(token, txtLeft).on_dark_gray()));
                                coloredRight.push((text.len(), self.GenerateColor(token, txtRight)));
                            } else {
                                coloredRight.push((text.len(), self.GenerateColor(token, text.as_str())));
                            }
                        } else {
                            coloredRight.push((text.len(), self.GenerateColor(token, text.as_str())));
                        }
                    } else {
                        let txt = &text[0..text.len() - (
                            currentCharNum + text.len() - self.cursor.1
                        )];
                        let leftSize = txt.len();
                        if self.highlighting && (self.cursorEnd.0 < self.cursor.0 || self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 < self.cursor.1) {
                            if self.cursorEnd.1 > currentCharNum && self.cursor.1 <= currentCharNum + leftSize && self.cursorEnd.1 - currentCharNum < text.len() &&
                                self.cursor.0 == self.cursorEnd.0
                            {
                                coloredLeft.push((
                                    self.cursorEnd.1 - currentCharNum,  // this is greater than the text length.....
                                    self.GenerateColor(token, &txt[..self.cursorEnd.1 - currentCharNum])
                                ));
                                coloredLeft.push((
                                    txt.len() - (self.cursorEnd.1 - currentCharNum),
                                    self.GenerateColor(token, &txt[self.cursorEnd.1 - currentCharNum..]).on_dark_gray()
                                ));
                            } else {
                                coloredLeft.push((
                                    txt.len(),
                                    self.GenerateColor(token, txt).on_dark_gray()
                                ));
                            }
                        } else {
                            coloredLeft.push((
                                txt.len(),
                                self.GenerateColor(token, txt)
                            ));
                        }
                        if editingCode {  coloredLeft.push((1, "|".to_string().white().bold()))  };
                        let txt = &text[
                            text.len() - (
                                currentCharNum + text.len() - self.cursor.1
                            )..text.len()
                        ];
                        if self.highlighting && (self.cursorEnd.0 > self.cursor.0 || self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 > self.cursor.1) {
                            if self.cursorEnd.1 > currentCharNum+leftSize && self.cursorEnd.1 < currentCharNum + text.len() {
                                coloredRight.push((
                                    self.cursorEnd.1 - (currentCharNum+leftSize),
                                    self.GenerateColor(token, &txt[..self.cursorEnd.1 - (currentCharNum+leftSize)]).on_dark_gray()
                                ));
                                coloredRight.push((
                                    txt.len() - (self.cursorEnd.1 - (currentCharNum+leftSize)),
                                    self.GenerateColor(token, &txt[self.cursorEnd.1 - (currentCharNum+leftSize)..])
                                ));
                            } else {
                                coloredRight.push((
                                    txt.len(),
                                    self.GenerateColor(token, txt).on_dark_gray()
                                ));
                            }
                        } else {
                            coloredRight.push((
                                txt.len(),
                                self.GenerateColor(token, txt)
                            ));
                        }
                    }
                } else if (self.cursorEnd.0 < self.cursor.0 || self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 < self.cursor.1) && self.highlighting {
                    if (lineNumber > self.cursorEnd.0 && lineNumber < self.cursor.0) ||
                        (lineNumber == self.cursor.0 && lineNumber == self.cursorEnd.0 &&
                        currentCharNum >= self.cursorEnd.1 && currentCharNum + text.len() <= self.cursor.1)
                    {
                        coloredLeft.push((text.len(), self.GenerateColor(token, text.as_str()).on_dark_gray()));
                    } else if currentCharNum+text.len() > self.cursorEnd.1 && currentCharNum < self.cursorEnd.1 && lineNumber == self.cursorEnd.0 {   // can't be equal to cursor line
                        let txtRight = &text[self.cursorEnd.1 - currentCharNum..];
                        let txtLeft = &text[..self.cursorEnd.1 - currentCharNum];
                        coloredLeft.push((text.len(), self.GenerateColor(token, txtLeft)));
                        coloredLeft.push((text.len(), self.GenerateColor(token, txtRight).on_dark_gray()));
                    } else if (lineNumber == self.cursor.0 && currentCharNum+text.len() <= self.cursor.1 ||
                        lineNumber == self.cursorEnd.0 && currentCharNum >= self.cursorEnd.1) && self.cursor.0 != self.cursorEnd.0
                    {
                        coloredLeft.push((text.len(), self.GenerateColor(token, text.as_str()).on_dark_gray()));
                    } else {
                        coloredLeft.push((text.len(), self.GenerateColor(token, text.as_str())));
                    }
                } else if self.highlighting {
                    if (lineNumber > self.cursor.0 && lineNumber < self.cursorEnd.0) ||
                        (lineNumber == self.cursorEnd.0 && lineNumber == self.cursor.0 &&
                        currentCharNum >= self.cursor.1 && currentCharNum + text.len() <= self.cursorEnd.1)
                    {
                        coloredLeft.push((text.len(), self.GenerateColor(token, text.as_str()).on_dark_gray()));
                    } else if currentCharNum+text.len() > self.cursorEnd.1 && currentCharNum <= self.cursorEnd.1 && lineNumber == self.cursorEnd.0 {   // can't be equal to cursor line
                        let txtRight = &text[self.cursorEnd.1 - currentCharNum..];
                        let txtLeft = &text[..self.cursorEnd.1 - currentCharNum];
                        coloredLeft.push((text.len(), self.GenerateColor(token, txtLeft).on_dark_gray()));
                        coloredLeft.push((text.len(), self.GenerateColor(token, txtRight)));
                    } else if (lineNumber == self.cursorEnd.0 && currentCharNum <= self.cursorEnd.1 ||
                            lineNumber == self.cursor.0 && currentCharNum >= self.cursor.1) && self.cursorEnd.0 != self.cursor.0 {
                                coloredLeft.push((text.len(), self.GenerateColor(token, text.as_str()).on_dark_gray()));
                    } else {
                        coloredLeft.push((text.len(), self.GenerateColor(token, text.as_str())));
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
            cursorEnd: (0, 0),
            highlighting: false,
            pauseScroll: 0,
        }
    }
}


#[derive(Debug)]
pub struct CodeTabs {
    pub tabFileNames: Vec <String>,
    pub tabs: Vec <CodeTab>,
    pub currentTab: usize,
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
                    cursorEnd: (0, 0),
                    highlighting: false,
                    pauseScroll: 0,
                }

            ],  // put a tab here or something idk
            currentTab: 0
        }
    }
}

