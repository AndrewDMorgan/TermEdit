
use ratatui::{
    layout::Rect,
    style::{Stylize, Modifier},
    text::{Line, Span},
};

use crate::Colors::Colors;
use crate::LuaScripts;
use crate::Tokens::*;


// the bounds from the screen edge at which the cursor will begin scrolling
const SCROLL_BOUNDS: usize = 12;
const CENTER_BOUNDS: usize = 0;


pub mod Edits {
    use crate::{CodeTab, LuaScripts};

    // private sense it's not needed elsewhere (essentially just a modified copy of handleHighlights...)
    async fn RemoveText (tab: &mut CodeTab, start: (usize, usize), end: (usize, usize), luaSyntaxHighlightScripts: &LuaScripts) {
        if end.0 == start.0 {
            tab.lines[end.0].replace_range(end.1..start.1, "");
            tab.RecalcTokens(end.0, 0, luaSyntaxHighlightScripts).await;
        } else {
            tab.lines[end.0].replace_range(end.1.., "");
            tab.RecalcTokens(end.0, 0, luaSyntaxHighlightScripts).await;
            tab.lines[end.0].replace_range(..start.1, "");
            tab.RecalcTokens(start.0, 0, luaSyntaxHighlightScripts).await;
            // go through any inbetween lines and delete them. Also delete one extra line so there aren't to blanks?
            let numBetween = start.0 - end.0 - 1;
            for _ in 0..numBetween {
                tab.lines.remove(end.0 + 1);
                tab.lineTokens.write().remove(end.0 + 1);
                tab.lineTokenFlags.write().remove(end.0 + 1);
            }
            // push the next line onto the first...
            let nextLine = tab.lines[end.0 + 1].clone();
            tab.lines[end.0].push_str(nextLine.as_str());
            tab.RecalcTokens(end.0, 0, luaSyntaxHighlightScripts).await;
            tab.lines.remove(end.0 + 1);
            tab.lineTokens.write().remove(end.0 + 1);
            tab.lineTokenFlags.write().remove(end.0 + 1);
        }
        
        tab.CreateScopeThread();
        //(tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);
        tab.cursor = end;
    }

    async fn AddText (tab: &mut CodeTab, start: (usize, usize), end: (usize, usize), text: &String, luaSyntaxHighlightScripts: &LuaScripts) {
        let splitText = text.split('\n');
        //let splitLength = splitText.clone().count() - 1;
        for (i, line) in splitText.enumerate() {
            if line.is_empty() {
                tab.lines.insert(end.0 + i, "".to_string());
                tab.lineTokens.write().insert(end.0 + i, vec![]);
                tab.lineTokenFlags.write().insert(end.0 + i, vec![]);
                tab.RecalcTokens(end.0 + i, 0, luaSyntaxHighlightScripts).await;
                continue;
            }

            if i == 0 {
                if end.1 >= tab.lines[end.0].len() {
                    tab.lines[end.0].push_str(line);
                } else {
                    tab.lines[end.0].insert_str(end.1, line);
                }
                tab.RecalcTokens(end.0, 0, luaSyntaxHighlightScripts).await;
            } else {
                tab.lines.insert(end.0 + i, line.to_string());
                tab.lineTokenFlags.write().insert(end.0 + i, vec![]);
                tab.lineTokens.write().insert(end.0 + i, vec![]);
                tab.RecalcTokens(end.0 + i, 0, luaSyntaxHighlightScripts).await;
            }
        }

        tab.CreateScopeThread();
        //(tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);
        tab.cursor = start;
    }
    
    #[derive(Debug)]
    pub struct Deletion {
        pub start: (usize, usize),
        pub end: (usize, usize),  // end is higher/before the start always
        pub text: String,
    }

    impl Deletion {
        pub async fn Undo (&self, tab: &mut CodeTab, luaSyntaxHighlightScripts: &LuaScripts) {
            AddText(tab, self.start, self.end, &self.text, luaSyntaxHighlightScripts).await;
        }
        
        pub async fn Redo (&self, tab: &mut CodeTab, luaSyntaxHighlightScripts: &LuaScripts) {
            RemoveText(tab, self.start, self.end, luaSyntaxHighlightScripts).await
        }
    }
    
    #[derive(Debug)]
    pub struct Addition {
        pub start: (usize, usize),
        pub end: (usize, usize),
        pub text: String,
    }

    impl Addition {
        pub async fn Undo (&self, tab: &mut CodeTab, luaSyntaxHighlightScripts: &LuaScripts) {
            RemoveText(tab, self.start, self.end, luaSyntaxHighlightScripts).await;
        }
        
        pub async fn Redo (&self, tab: &mut CodeTab, luaSyntaxHighlightScripts: &LuaScripts) {
            AddText(tab, self.start, self.end, &self.text, luaSyntaxHighlightScripts).await;
        }
    }

    #[derive(Debug)]
    pub struct NewLine {
        pub position: (usize, usize),
    }

    impl NewLine {
        pub async fn Undo (&self, tab: &mut CodeTab, luaSyntaxHighlightScripts: &LuaScripts) {
            let text = tab.lines.remove(self.position.0 + 1);
            tab.lineTokens.write().remove(self.position.0 + 1);
            tab.lineTokenFlags.write().remove(self.position.0 + 1);
            tab.lines[self.position.0].push_str(text.as_str());
            tab.RecalcTokens(self.position.0, 0, luaSyntaxHighlightScripts).await;

            tab.cursor.0 = self.position.0.saturating_sub(1);
            tab.cursor.1 = tab.lines[tab.cursor.0].len();
            tab.CreateScopeThread();
            //(tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);
        }
        
        pub async fn Redo (&self, tab: &mut CodeTab, luaSyntaxHighlightScripts: &LuaScripts) {
            let rightText = tab.lines[self.position.0]
                .split_off(self.position.1);
            tab.lines.insert(self.position.0 + 1, rightText.to_string());
            tab.lineTokens.write().insert(self.position.0 + 1, vec![]);
            tab.lineTokenFlags.write().insert(self.position.0 + 1, vec![]);
            tab.RecalcTokens(self.position.0 + 1, 0, luaSyntaxHighlightScripts).await;
            tab.RecalcTokens(self.position.0, 0, luaSyntaxHighlightScripts).await;

            tab.cursor = (
                self.position.0 + 1,
                0
            );
            tab.CreateScopeThread();
            //(tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);
        }
    }

    #[derive(Debug)]
    pub struct RemoveLine {
        pub position: (usize, usize),
    }

    impl RemoveLine {
        pub async fn Undo (&self, tab: &mut CodeTab, luaSyntaxHighlightScripts: &LuaScripts) {
            let rightText = tab.lines[self.position.0]
                .split_off(self.position.1);
            tab.lines.insert(self.position.0 + 1, rightText.to_string());
            tab.lineTokens.write().insert(self.position.0 + 1, vec![]);
            tab.lineTokenFlags.write().insert(self.position.0 + 1, vec![]);
            tab.RecalcTokens(self.position.0 + 1, 0, luaSyntaxHighlightScripts).await;
            tab.RecalcTokens(self.position.0, 0, luaSyntaxHighlightScripts).await;

            tab.cursor = (
                self.position.0 + 1,
                0
            );
            tab.CreateScopeThread();
            //(tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);
        }
        
        pub async fn Redo (&self, tab: &mut CodeTab, luaSyntaxHighlightScripts: &LuaScripts) {
            let text = tab.lines.remove(self.position.0 + 1);
            tab.lineTokens.write().remove(self.position.0 + 1);
            tab.lineTokenFlags.write().remove(self.position.0 + 1);
            tab.lines[self.position.0].push_str(text.as_str());
            tab.RecalcTokens(self.position.0, 0, luaSyntaxHighlightScripts).await;

            tab.cursor.0 = self.position.0.saturating_sub(1);
            tab.cursor.1 = tab.lines[tab.cursor.0].len();
            tab.CreateScopeThread();
            //(tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);
        }
    }
    
    #[derive(Debug)]
    pub enum Edit {
        Deletion (Deletion),
        Addition (Addition),
        NewLine (NewLine),
        RemoveLine (RemoveLine),
    }
}


// only access the inner value explicitly editing the value, then return ownership
// otherwise the element will block usage of the main item in the codeTab, which
// would block the main thread (aka not good)
/*type ScopeHandle = std::thread::JoinHandle< Box <dyn Fn(
        std::sync::Arc < parking_lot::RwLock <ScopeNode>>,
        std::sync::Arc < parking_lot::RwLock <Vec <Vec <usize>>>>,
        std::sync::Arc < parking_lot::RwLock <Vec <Vec <usize>>>>,
        crossbeam::channel::Sender <bool>,
) -> () >>;*/

// this isn't nearly as long as I thought it would be lol (too lazy to inline it)
type ScopeHandle = std::thread::JoinHandle <()>;

#[derive(Debug)]
pub struct CodeTab {
    pub cursor: (usize, usize),  // line pos, char pos inside line
    pub lines: Vec <String>,
    pub lineTokens: std::sync::Arc <parking_lot::RwLock <Vec <Vec <LuaTuple>>>>,
    // points to the index of the scope (needs adjusting as the tree is modified)
    pub scopeJumps: std::sync::Arc <parking_lot::RwLock <Vec <Vec <usize>>>>,
    pub scopes: std::sync::Arc <parking_lot::RwLock <ScopeNode>>,
    pub linearScopes: std::sync::Arc <parking_lot::RwLock <Vec <Vec <usize>>>>,

    pub scrolled: usize,
    pub mouseScrolled: isize,
    pub mouseScrolledFlt: f64,

    pub name: String,
    pub fileName: String,
    pub cursorEnd: (usize, usize),  // for text highlighting
    pub highlighting: bool,
    pub pauseScroll: u128,

    pub searchIndex: usize,
    pub searchTerm: String,

    pub changeBuffer: Vec <Vec <Edits::Edit>>,
    pub redoneBuffer: Vec <Vec <Edits::Edit>>,  // stores redo's (cleared if undone then edited)
    pub pinedLines: Vec <usize>,  // todo figure out a way to have a color for the pinned points (maybe an enum?--or just a color....)

    pub outlineKeywords: std::sync::Arc <parking_lot::RwLock <Vec <OutlineKeyword>>>,
    // each line can have multiple flags depending on the depth (each line has a set for each token......)
    pub lineTokenFlags: std::sync::Arc <parking_lot::RwLock <Vec <Vec < Vec <LineTokenFlags>>>>>,

    pub scopeGenerationHandles: Vec <(ScopeHandle, crossbeam::channel::Receiver <bool>)>,
}

impl CodeTab {

    pub fn CreateScopeThread (&mut self) {
        // offsetting from existing calculations (staggering the computation (1/4 second per thread)
        // if there's at least 2, there should be one still queued and not running
        // so it shouldn't cause outdated information. This value may need adjustment?
        if self.scopeGenerationHandles.len() >= 2 {  return;  }
        // the time of 1/4 seconds is completely random. Seems like it might be reasonable though
        let timeOffset = self.scopeGenerationHandles.len() as u64 * 250;

        // the memory leak is independent of the number of threads or when they're joined (fixed!!!)
        //if !self.scopeGenerationHandles.is_empty() {  return;  }
        let (sender, receiver) = crossbeam::channel::bounded(1);
        let scopeClone = std::sync::Arc::clone(&self.scopes);
        let jumpsClone = std::sync::Arc::clone(&self.scopeJumps);
        let linearClone = std::sync::Arc::clone(&self.linearScopes);

        let lineTokensClone = std::sync::Arc::clone(&self.lineTokens);
        let lineFlagsClone = std::sync::Arc::clone(&self.lineTokenFlags);
        let outlineClone = std::sync::Arc::clone(&self.outlineKeywords);
        self.scopeGenerationHandles.push((
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(timeOffset));

                let (newScopes, newJumps, newLinear) =
                    GenerateScopes(&lineTokensClone, &lineFlagsClone, &outlineClone);

                // waiting for a free spot to let it write (and immediately dropping it)
                let mut writeGuard = scopeClone.write();
                *writeGuard = newScopes;
                // making sure it's not blocking the main thread
                // if something's trying to read, it has to wait for
                // this to be called/finished
                drop(writeGuard);

                // waiting for a free spot to let it write (and immediately dropping it)
                let mut writeGuard = jumpsClone.write();
                *writeGuard = newJumps;
                // making sure it's not blocking the main thread
                // if something's trying to read, it has to wait for
                // this to be called/finished
                drop(writeGuard);

                // waiting for a free spot to let it write (and immediately dropping it)
                let mut writeGuard = linearClone.write();
                *writeGuard = newLinear;
                // making sure it's not blocking the main thread
                // if something's trying to read, it has to wait for
                // this to be called/finished
                drop(writeGuard);

                // sending a message to join the thread
                sender.send(true).unwrap();
            }),
            receiver
        ));
        //let (thread, rec) = self.scopeGenerationHandles.pop().unwrap();
        //thread.join().unwrap();
    }

    pub fn CheckScopeThreads (&mut self) {
        // the indexes of finished threads so they can be removed and joined
        let mut finishedThreads: Vec <usize> = vec!();

        // checking all thread channels
        for i in 0..self.scopeGenerationHandles.len() {
            let thread = &self.scopeGenerationHandles[i];
            if thread.1.try_recv().is_ok() {
                finishedThreads.push(i);
            }
        }

        // joining necessary threads
        // the indexes are in descending order (ascending order but reversed with .rev)
        // because of this, the indexes shouldn't interfere
        // with the following remove operations
        for index in finishedThreads.into_iter().rev() {
            let thread = self.scopeGenerationHandles.remove(index);
            // ignoring any errors for now
            // all the memory of the thread should be killed
            // all the memory is shared so it should be fine as is
            let _ = thread.0.join();
        }
    }

    pub fn GetCurrentToken (&self, tokenOutput: &mut Vec <String>) {
        let mut accumulate = 0;
        let lineTokensRead = self.lineTokens.read();
        for (tokenIndex, token) in lineTokensRead[self.cursor.0].iter().enumerate() {
            // the cursor can be just right of it, in it, but not just left
            if (accumulate + token.text.len()) >= self.cursor.1 && self.cursor.1 > accumulate {
                tokenOutput.push(token.text.clone());
                for index in (0..tokenIndex).rev() {
                    if matches!(lineTokensRead[self.cursor.0][index].text.as_str(),
                        " " | "," | "(" | ")" | ";")
                        {  break;  }
                    if index > 1 &&
                        lineTokensRead[self.cursor.0][index].text == ":" &&
                        lineTokensRead[self.cursor.0][index - 1].text == ":"
                    {
                        tokenOutput.push(lineTokensRead[self.cursor.0][index - 2].text.clone());
                    } else if index > 0 && lineTokensRead[self.cursor.0][index].text == "." {
                        tokenOutput.push(lineTokensRead[self.cursor.0][index - 1].text.clone());
                    }
                }
                return;
            }
            accumulate += token.text.len();
        } // lineTokensRead is dropped naturally
    }

    // doesn't update the tokens or scopes; requires that to be done elsewhere
    pub fn RemoveCurrentToken_NonUpdate (&mut self) {
        let mut accumulate = 0;
        let lineTokensRead = self.lineTokens.read();
        for token in lineTokensRead[self.cursor.0].iter() {
            // the cursor can be just right of it, in it, but not just left
            if (accumulate + token.text.len()) >= self.cursor.1 && self.cursor.1 > accumulate {
                self.lines[self.cursor.0].replace_range(accumulate..accumulate+token.text.len(), "");
                self.cursor.1 = accumulate;
                return;
            }
            accumulate += token.text.len();
        }  // lineTokensRead is naturally dropped
    }

    pub async fn Undo (&mut self, luaSyntaxHighlightScripts: &LuaScripts) {
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        self.highlighting = false;

        if let Some(edits) = self.changeBuffer.pop() {
            for edit in &edits {
                match edit {
                    Edits::Edit::Addition (action) => {
                        action.Undo( self, luaSyntaxHighlightScripts).await;
                    },
                    Edits::Edit::Deletion (action) => {
                        action.Undo(self, luaSyntaxHighlightScripts).await;
                    },
                    Edits::Edit::RemoveLine (action) => {
                        action.Undo(self, luaSyntaxHighlightScripts).await;
                    },
                    Edits::Edit::NewLine (action) => {
                        action.Undo(self, luaSyntaxHighlightScripts).await;
                    },
                }
            }
            self.redoneBuffer.push(edits);
        }
    }

    pub async fn Redo (&mut self, luaSyntaxHighlightScripts: &LuaScripts) {
        // resetting a bunch of things
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        self.highlighting = false;

        if let Some(edits) = self.redoneBuffer.pop() {
            for edit in &edits {
                match edit {
                    Edits::Edit::Addition (action)      => {
                        action.Redo(self, luaSyntaxHighlightScripts).await;
                    },
                    Edits::Edit::Deletion (action)      => {
                        action.Redo(self, luaSyntaxHighlightScripts).await;
                    },
                    Edits::Edit::RemoveLine (action) => {
                        action.Redo(self, luaSyntaxHighlightScripts).await;
                    },
                    Edits::Edit::NewLine (action)       => {
                        action.Redo(self, luaSyntaxHighlightScripts).await;
                    },
                }
            }
            self.changeBuffer.push(edits);
        }
    }

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
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        self.cursor.1 = std::cmp::min (
            self.cursor.1,
            self.lines[self.cursor.0].len()
        );
        
        // walking back till no longer on a space
        while self.cursor.1 > 0 && self.lines[self.cursor.0]
            .get(self.cursor.1-1..self.cursor.1)
            .unwrap_or("") == " "
        {
            self.cursor.1 -= 1;
        }
        
        let mut totalLine = String::new();
        let lineTokensRead = self.lineTokens.read();
        for token in &lineTokensRead[self.cursor.0] {
            if totalLine.len() + token.text.len() >= self.cursor.1 {
                self.cursor.1 = totalLine.len();
                return;
            }
            totalLine.push_str(token.text.as_str());
        }  // lineTokensRead is naturally dropped
    }

    pub fn FindTokenPosLeft (&mut self) -> usize {
        self.cursor.1 = std::cmp::min (
            self.cursor.1,
            self.lines[self.cursor.0].len()
        );
        let mut newCursor = self.cursor.1;

        while newCursor > 0 && self.lines[self.cursor.0]
            .get(newCursor-1..newCursor)
            .unwrap_or("") == " "
        {
            newCursor -= 1;
        }

        let mut totalLine = String::new();
        let lineTokensRead = self.lineTokens.read();
        for token in &lineTokensRead[self.cursor.0] {
            if totalLine.len() + token.text.len() >= newCursor {
                newCursor = totalLine.len();
                break;
            }
            totalLine.push_str(token.text.as_str());
        }
        drop(lineTokensRead);

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
            self.lines[self.cursor.0]
                .get(newCursor..newCursor + 1)
                .unwrap_or("") == " "
        {
            newCursor += 1;
        }

        let mut totalLine = String::new();
        let lineTokensRead = self.lineTokens.read();
        for token in &lineTokensRead[self.cursor.0] {
            if totalLine.len() + token.text.len() > newCursor {
                newCursor = totalLine.len() + token.text.len();
                break;
            }
            totalLine.push_str(token.text.as_str());
        }
        drop(lineTokensRead);

        newCursor - self.cursor.1
    }
    
    pub fn MoveCursorRightToken (&mut self) {
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        let mut totalLine = String::new();
        let lineTokensRead = self.lineTokens.read();
        for token in &lineTokensRead[self.cursor.0] {
            if token.text != " " && totalLine.len() + token.text.len() > self.cursor.1 {
                self.cursor.1 = totalLine.len() + token.text.len();
                return;
            }
            totalLine.push_str(token.text.as_str());
        }  // lineTokensRead is naturally dropped
    }
    
    pub fn MoveCursorLeft (&mut self, amount: usize, highlight: bool) {
        if self.highlighting && !highlight && self.cursor.0 > self.cursorEnd.0 ||
            self.cursor.1 > self.cursorEnd.1 && !highlight && self.highlighting
        {
            (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
            self.highlighting = false;
            return;
        } else if self.highlighting && !highlight {
            self.highlighting = false;
            return;
        }
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
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
            self.cursor.1 < self.cursorEnd.1 && self.cursor.0 == self.cursorEnd.1
                && !highlight && self.highlighting
        {
            (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
            self.highlighting = false;
            return;
        } else if self.highlighting && !highlight {
            self.highlighting = false;
            return;
        }
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        if self.cursor.1 >= self.lines[self.cursor.0].len() &&
            self.cursor.0 < self.lines.len() - 1
        {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
            return;
        }

        self.cursor = (
            self.cursor.0,
            self.cursor.1.saturating_add(amount)
        );
    }

    pub async fn InsertChars (&mut self, chs: String, luaSyntaxHighlightScripts: &LuaScripts) {
        self.redoneBuffer.clear();
        let mut changeBuff = vec!();

        // doesn't need to exit bc/ chars should still be added
        self.HandleHighlight(&mut changeBuff, luaSyntaxHighlightScripts).await;

        let preCursor = self.cursor;

        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
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

        changeBuff.insert(0,
            Edits::Edit::Addition(Edits::Addition {
                start: self.cursor,
                end: preCursor,
                text: chs.clone()
            })
        );
        self.changeBuffer.push(
            changeBuff
        );

        self.RecalcTokens(self.cursor.0, 0, luaSyntaxHighlightScripts).await;

        self.CreateScopeThread();
        //(self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens, &self.lineTokenFlags, &mut self.outlineKeywords);
    }

    pub async fn UnIndent (&mut self, luaSyntaxHighlightScripts: &LuaScripts) {
        self.redoneBuffer.clear();
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        // checking for 4 spaces at the start
        if let Some(charSet) = &self.lines[self.cursor.0].get(..4) {
            if *charSet == "    " {
                self.changeBuffer.push(
                    vec![
                        Edits::Edit::Deletion(Edits::Deletion {
                            start: (self.cursor.0, 3),  // I think it should be 3
                            end: (self.cursor.0, 0),
                            text: "    ".to_string()
                        })
                    ]
                );

                for _ in 0..4 {  self.lines[self.cursor.0].remove(0);  }
                self.cursor.1 = self.cursor.1.saturating_sub(4);

                self.RecalcTokens(self.cursor.0, 0, luaSyntaxHighlightScripts).await;

                self.CreateScopeThread();
                //(self.scopes, self.scopeJumps, self.linearScopes) =
                //    GenerateScopes(&self.lineTokens, &self.lineTokenFlags, &mut self.outlineKeywords);
            }
        }
    }

    pub fn CursorUp (&mut self, highlight: bool) {
        if self.highlighting && !highlight && self.cursor.0 > self.cursorEnd.0 ||
            self.cursor.1 > self.cursorEnd.1 && !highlight && self.highlighting
        {
            (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
            self.highlighting = false;
        } else if self.highlighting && !highlight {
            self.highlighting = false;
        }
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        self.cursor = (
            self.cursor.0.saturating_sub(1),
            self.cursor.1
        );
    }

    pub fn CursorDown (&mut self, highlight: bool) {
        if self.highlighting && !highlight && self.cursor.0 < self.cursorEnd.0 ||
            self.cursor.1 < self.cursorEnd.1 && self.cursor.0 == self.cursorEnd.1
                && !highlight && self.highlighting
        {
            (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
            self.highlighting = false;
        } else if self.highlighting && !highlight {
            self.highlighting = false;
        }

        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
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

    pub async fn LineBreakIn (&mut self, highlight: bool, luaSyntaxHighlightScripts: &LuaScripts) {
        self.redoneBuffer.clear();
        self.changeBuffer.push(
            vec![
                Edits::Edit::NewLine(Edits::NewLine {
                    position: self.cursor
                })
            ]
        );
        
        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        let length = self.lines[self.cursor.0].len();

        if length == 0 {
            self.lines.insert(self.cursor.0, "".to_string());
            let mut lineTokensWrite = self.lineTokens.write();
            lineTokensWrite[self.cursor.0].clear();
            lineTokensWrite.insert(self.cursor.0, vec!());
            drop(lineTokensWrite);  // the .write is dropped (writes can back up all the reads)
            self.lineTokenFlags.write().insert(self.cursor.0, vec!());

            self.CreateScopeThread();
            //(self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens, &self.lineTokenFlags, &mut self.outlineKeywords);

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
        self.lineTokens.write().insert(
            self.cursor.0 + 1,
            vec!(),
        );
        self.lineTokenFlags.write().insert(self.cursor.0 + 1, vec!());
        
        self.RecalcTokens(self.cursor.0, 0, luaSyntaxHighlightScripts).await;
        self.RecalcTokens(self.cursor.0 + 1, 0, luaSyntaxHighlightScripts).await;
        self.cursor.1 = 0;
        self.CursorDown(highlight);

        self.CreateScopeThread();
        //(self.scopes, self.scopeJumps, self.linearScopes) = GenerateScopes(&self.lineTokens, &self.lineTokenFlags, &mut self.outlineKeywords);

    }

    pub async fn HandleHighlight (&mut self, changeBuff: &mut Vec <Edits::Edit>, luaSyntaxHighlightScripts: &LuaScripts) -> bool {
        self.redoneBuffer.clear();
        if self.highlighting && self.cursorEnd != self.cursor {
            if self.cursorEnd.0 < self.cursor.0 ||
                 self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 < self.cursor.1
            {
                if self.cursorEnd.0 == self.cursor.0 {
                    changeBuff.push(Edits::Edit::Deletion(Edits::Deletion {
                        start: self.cursor,
                        end: self.cursorEnd,
                        text: self.lines[self.cursorEnd.0]
                            .get(self.cursorEnd.1..self.cursor.1)
                            .unwrap_or("")
                            .to_string()
                    }));
                    self.lines[self.cursorEnd.0]
                        .replace_range(self.cursorEnd.1..self.cursor.1, "");
                    self.RecalcTokens(self.cursor.0, 0, luaSyntaxHighlightScripts).await;
                } else {
                    let mut accumulative = String::new();
                    accumulative.push_str(
                        self.lines[self.cursorEnd.0]
                            .get(self.cursorEnd.1..).unwrap_or("")
                    );
                    accumulative.push('\n');
                    self.lines[self.cursorEnd.0]
                        .replace_range(self.cursorEnd.1.., "");
                    self.RecalcTokens(self.cursorEnd.0, 0, luaSyntaxHighlightScripts).await;

                    // go through any inbetween lines and delete them. Also delete one extra line so there aren't to blanks?
                    let numBetween = self.cursor.0 - self.cursorEnd.0 - 1;
                    for _ in 0..numBetween {
                        accumulative.push_str(
                            self.lines[self.cursorEnd.0 + 1].clone().as_str()
                        );
                        accumulative.push('\n');
                        self.lines.remove(self.cursorEnd.0 + 1);
                        self.lineTokens.write().remove(self.cursorEnd.0 + 1);
                        self.lineTokenFlags.write().remove(self.cursorEnd.0 + 1);
                    }

                    accumulative.push_str(
                        self.lines[self.cursorEnd.0 + 1]
                            .get(..self.cursor.1).unwrap_or("")
                    );
                    accumulative.push('\n');
                    self.lines[self.cursorEnd.0 + 1]
                        .replace_range(..self.cursor.1, "");
                    self.RecalcTokens(self.cursor.0, 0, luaSyntaxHighlightScripts).await;
                    // push the next line onto the first...
                    let nextLine = self.lines[self.cursorEnd.0 + 1].clone();
                    accumulative.push_str(
                        nextLine.clone().as_str()
                    );  // does a \n go right after this? Or is it not needed??????
                    self.lines[self.cursorEnd.0].push_str(nextLine.as_str());
                    self.RecalcTokens(self.cursorEnd.0, 0, luaSyntaxHighlightScripts).await;
                    self.lines.remove(self.cursorEnd.0 + 1);
                    self.lineTokens.write().remove(self.cursorEnd.0 + 1);
                    self.lineTokenFlags.write().remove(self.cursorEnd.0 + 1);

                    changeBuff.push(Edits::Edit::Deletion(Edits::Deletion {
                        start: self.cursor,
                        end: self.cursorEnd,
                        text: accumulative
                    }));
                    // I have no clue if this is actually correct or not; mostly the cursor position stuff
                    changeBuff.push(Edits::Edit::NewLine(Edits::NewLine {
                        position: (
                            self.cursor.0 + 1,
                            0
                        )
                    }));
                }
                
                self.highlighting = false;
                self.cursor = self.cursorEnd;
                self.CreateScopeThread();
                //(self.scopes, self.scopeJumps, self.linearScopes) =
                //    GenerateScopes(&self.lineTokens, &self.lineTokenFlags, &mut self.outlineKeywords);
                return true;
            } else {
                // swapping the cursor and ending points so the other calculations work
                (self.cursor, self.cursorEnd) = (self.cursorEnd, self.cursor);
                return Box::pin(async move {
                    self.HandleHighlight(changeBuff, luaSyntaxHighlightScripts).await
                }).await;
            }
        } false
    }

    pub fn GetSelection (&self) -> String {
        let mut occumulation = String::new();

        if self.highlighting && self.cursor != self.cursorEnd {
            if self.cursorEnd.0 == self.cursor.0 {
                if self.cursorEnd.1 < self.cursor.1 {  // cursor on the smae line
                    let selection =
                        &self.lines[self.cursor.0][self.cursorEnd.1..self.cursor.1];
                    occumulation.push_str(selection);
                } else {
                    let selection =
                        &self.lines[self.cursor.0][self.cursor.1..self.cursorEnd.1];
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
    pub async fn DelChars (&mut self, numDel: usize, cursorOffset: usize, luaSyntaxHighlightScripts: &LuaScripts) {
        self.redoneBuffer.clear();

        // deleting characters from scrolling
        let mut changeBuff = vec!();
        if self.HandleHighlight(&mut changeBuff, luaSyntaxHighlightScripts).await {
            self.changeBuffer.push(changeBuff);
            return;
        }

        let preCursor = self.cursor;
        let mut deletedText = String::new();

        self.scrolled = std::cmp::max(
            self.mouseScrolledFlt as isize + self.scrolled as isize,
            0
        ) as usize;
        self.mouseScrolled = 0;
        self.mouseScrolledFlt = 0.0;
        let length = self.lines[self.cursor.0]
            .len();

        if self.cursor.1 < numDel && cursorOffset == 0 && self.lines.len() > 1 {
            // the remaining text
            deletedText.push_str(
                self.lines[self.cursor.0]
                    .get(..self.cursor.1)
                    .unwrap_or("")
            );
            let remaining = self.lines[self.cursor.0].split_off(self.cursor.1);

            self.lines.remove(self.cursor.0);
            self.lineTokens.write().remove(self.cursor.0);
            self.lineTokenFlags.write().remove(self.cursor.0);
            self.cursor.0 = self.cursor.0.saturating_sub(1);
            self.cursor.1 = self.lines[self.cursor.0].len();

            self.lines[self.cursor.0].push_str(remaining.as_str());
            self.RecalcTokens(self.cursor.0, 0, luaSyntaxHighlightScripts).await;

            changeBuff.insert(0,
                Edits::Edit::Deletion(Edits::Deletion{
                    start: self.cursor,
                    end: preCursor,
                    text: deletedText
                })
            );
            changeBuff.insert(1,
                Edits::Edit::RemoveLine(Edits::RemoveLine{
                    position: (
                        preCursor.0,
                        0
                    )
                })
            );
            self.changeBuffer.push(
                changeBuff
            );

            self.CreateScopeThread();
            //self.linearScopes) =
            //    GenerateScopes(&self.lineTokens, &self.lineTokenFlags, &mut self.outlineKeywords);

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

        deletedText.push_str(
            self.lines[self.cursor.0]
                .get(newCursor
                    ..
                    std::cmp::min(
                        self.cursor.1.saturating_add(cursorOffset),
                        length
                )
            ).unwrap_or("")
        );
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

        changeBuff.insert(0,
            Edits::Edit::Deletion(Edits::Deletion{
                start: preCursor,
                end: self.cursor,
                text: deletedText
            })
        );
        self.changeBuffer.push(
            changeBuff
        );

        self.RecalcTokens(self.cursor.0, 0, luaSyntaxHighlightScripts).await;

        self.CreateScopeThread();
        //(self.scopes, self.scopeJumps, self.linearScopes) =
        //    GenerateScopes(&self.lineTokens, &self.lineTokenFlags, &mut self.outlineKeywords);
    }

    pub async fn RecalcTokens (&mut self,
                         lineNumber: usize,
                         recursed: usize,
                         luaSyntaxHighlightScripts: &LuaScripts
    ) {
        if lineNumber >= self.lines.len() {  return;  }
        let lineTokenFlagsRead = self.lineTokenFlags.read();
        let containedComment =
            lineTokenFlagsRead[lineNumber]
                .get(lineTokenFlagsRead[lineNumber].len().saturating_sub(1))
                .unwrap_or(&vec![])
                .contains(&LineTokenFlags::Comment);
        let previousEnding = lineTokenFlagsRead[lineNumber].get(
            lineTokenFlagsRead[lineNumber].len().saturating_sub(1)
        ).unwrap_or(&vec!()).clone();
        drop(lineTokenFlagsRead);
        self.lineTokens.write()[lineNumber].clear();

        let ending = self.fileName.split('.').last().unwrap_or("");
        let newTokens = GenerateTokens(
                    self.lines[lineNumber].clone(),
                    ending, &mut self.lineTokenFlags,
                    lineNumber,
                    &mut self.outlineKeywords,
                    luaSyntaxHighlightScripts
        ).await;
        // not being given up? crashing here
        self.lineTokens.write()[lineNumber] = newTokens;

        let lineTokenFlagsRead = self.lineTokenFlags.read();
        let currentFlags = lineTokenFlagsRead[lineNumber][
            lineTokenFlagsRead[lineNumber].len() - 1
        ].clone();
        drop(lineTokenFlagsRead);

        let empty = currentFlags.is_empty();
        if (lineNumber < self.lines.len() - 1 && !empty &&
                previousEnding != currentFlags ||
                empty && previousEnding.len() > 0) &&
            (
                recursed < 250 && (
                    containedComment || currentFlags.contains(&LineTokenFlags::Comment)
                ) || recursed < 25
            ) {
            
            Box::pin(async move {
                self.RecalcTokens(lineNumber + 1, recursed + 1, luaSyntaxHighlightScripts).await;  // cascading any changes further down the file (kinda slow)
            }).await;
        }
        
        // recalculating variables, methods, etc...
    }

    pub fn GenerateColor <'a> (&self, token: &TokenType, text: String, colorMode: &Colors::ColorMode) -> Span <'a> {
        match token {
            TokenType::Bracket => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::SquirlyBracket => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Parentheses => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Variable => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Member => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Object => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Function => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Method => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                ).italic()  // works for now ig
            },
            TokenType::Number => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Logic => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Math => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Assignment => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Endl => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Macro => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                ).italic().bold().underlined()
            },
            TokenType::Const => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                ).italic()
            },
            TokenType::Barrow => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                ).italic()
            },
            TokenType::Lifetime => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                ).underlined().bold()
            },
            TokenType::String => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Comment | TokenType::CommentLong => {
                if text == "todo" || text == "!" || text == "Todo" ||
                    text == "error" || text == "condition" ||
                    text == "conditions" || text == "fix" {
                        text.fg(
                            *colorMode.colorBindings
                                .syntaxHighlighting
                                .get(&(&token, &colorMode.colorType))
                                .expect("Error.... no color found")
                        ).underlined()
                }  // basic but it kinda does stuff idk
                else {
                    text.fg(
                        *colorMode.colorBindings
                            .syntaxHighlighting
                            .get(&(&token, &colorMode.colorType))
                            .expect("Error.... no color found")
                    )
                }
            },
            TokenType::Null => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Primitive => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                )
            },
            TokenType::Keyword => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                ).bold().underlined()
            },
            TokenType::Unsafe => {
                text.fg(
                    *colorMode.colorBindings
                        .syntaxHighlighting
                        .get(&(&token, &colorMode.colorType))
                        .expect("Error.... no color found")
                ).italic().underlined().on_dark_gray().bold()
            },
            TokenType::Grayed => {
                text.white().dim().italic()
            }
        }
    }

    pub fn GetScrolledText <'a> (&mut self, area: Rect,
                                 editingCode: bool,
                                 colorMode: &Colors::ColorMode,
                                 suggested: &'a String,  // the suggested auto-complete (for inline rendering)
) -> Vec <Line> {
        // using the known area to adjust the scrolled position (even though this can now be done else wise..... too lazy to move it)
        let currentTime = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Time went backwards...")
            .as_millis();
        if currentTime.saturating_sub(self.pauseScroll) > 125 {
            if self.scrolled + SCROLL_BOUNDS >= self.cursor.0 {
                if self.scrolled
                    .saturating_sub(CENTER_BOUNDS) >=
                    self.cursor.0 && !self.highlighting
                {
                    let center = std::cmp::min(
                        self.cursor.0
                            .saturating_sub((area.height as usize)
                            .saturating_sub(10) / 2),
                        self.lines.len() - 1
                    );
                    self.scrolled = center;
                } else {
                    self.scrolled = self.cursor.0.saturating_sub(SCROLL_BOUNDS);
                    if self.highlighting {  // making sure the highlighting doesn't scroll at light speed
                        std::thread::sleep(std::time::Duration::from_millis(25));
                    }
                }
            }
            if (self.scrolled + area.height as usize - 12)
                .saturating_sub(SCROLL_BOUNDS) <= self.cursor.0
            {
                if self.scrolled + area.height as usize + CENTER_BOUNDS <=
                    self.cursor.0 && !self.highlighting
                {
                    let center = std::cmp::min(
                        self.cursor.0
                            .saturating_sub((area.height as usize)
                            .saturating_sub(10) / 2),
                        self.lines.len() - 1
                    );
                    self.scrolled = center;
                } else {
                    self.scrolled = (self.cursor.0 + SCROLL_BOUNDS)
                        .saturating_sub(area.height as usize - 12);
                    if self.highlighting {  // making sure the highlighting doesn't scroll at light speed
                        std::thread::sleep(std::time::Duration::from_millis(25));
                    }
                }
            }
        }

        let scroll = std::cmp::max(
            self.scrolled as isize + self.mouseScrolled,
            0
        ) as usize;
        
        let mut tabText = vec![];
        
        let mut i = 0;
        for lineNumber in scroll..(scroll + area.height as usize - 10) {
            if lineNumber >= self.lines.len() { continue; }

            let mut lineNumberText = format!("{}: ",
                                             (lineNumber as isize - self.cursor.0 as isize)
                                                 .unsigned_abs());
            if self.cursor.0 == lineNumber {
                lineNumberText = format!("{}: ", lineNumber + 1);
            }

            // adjust this for the total length of the file so everything is held to the same line length
            let totalSize = (self.lines.len()).to_string().len() + 1;  // number of digits + 2usize;
            for _ in 0..totalSize {
                if lineNumberText.len() <= totalSize {
                    lineNumberText.push(' ');
                }
            }

            let mut coloredLeft: Vec<(usize, Span)> = vec!();
            let mut coloredRight: Vec<(usize, Span)> = vec!();

            if lineNumber == self.cursor.0 {
                coloredLeft.push((lineNumberText.len(),
                                  lineNumberText
                                      .red()
                                      .bold()
                                      .add_modifier(Modifier::UNDERLINED)));
            } else {
                coloredLeft.push((lineNumberText.len(), lineNumberText.gray().italic()));
            }
            
            let mut currentCharNum = 0;
            let lineTokensRead = self.lineTokens.read();
            for token in &lineTokensRead[lineNumber] {
                let tokenClone = token.text.clone();
                let tokenCloneStr = tokenClone.clone();
                //let tokenCloneStr: &'a str = &tokenClone[..];
                if lineNumber == self.cursor.0 && currentCharNum + token.text.len() > self.cursor.1 {
                    if currentCharNum >= self.cursor.1 {
                        if currentCharNum == self.cursor.1 && editingCode {
                            coloredLeft.push((1, "|".to_string().white().bold()));
                        }
                        if self.highlighting && (self.cursorEnd.0 > self.cursor.0 || self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 > self.cursor.1) {
                            if self.highlighting && (lineNumber == self.cursorEnd.0 && currentCharNum + token.text.len() <= self.cursorEnd.1 ||
                                lineNumber == self.cursor.0 && currentCharNum >= self.cursor.1) && self.cursor.0 != self.cursorEnd.0 ||
                                (lineNumber > self.cursor.0 && lineNumber < self.cursorEnd.0) ||
                                (lineNumber == self.cursorEnd.0 && lineNumber == self.cursor.0 &&
                                    currentCharNum >= self.cursor.1 && currentCharNum + token.text.len() <= self.cursorEnd.1)
                            {
                                coloredRight.push(
                                    (token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode)
                                        .on_dark_gray())
                                );
                            } else if self.highlighting && currentCharNum + token.text.len() > self.cursorEnd.1 && currentCharNum < self.cursorEnd.1 && lineNumber == self.cursorEnd.0 {   // can't be equal to cursor line
                                let txtRight = tokenClone[self.cursorEnd.1 - currentCharNum..].to_string();
                                let txtLeft = tokenClone[..self.cursorEnd.1 - currentCharNum].to_string();
                                coloredRight.push(
                                    (token.text.len(), self.GenerateColor(&token.token, txtLeft, colorMode)
                                        .on_dark_gray())
                                );
                                coloredRight.push(
                                    (token.text.len(), self.GenerateColor(&token.token, txtRight, colorMode))
                                );
                            } else {
                                coloredRight.push(
                                    (token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode))
                                );
                            }
                        } else {
                            coloredRight.push(
                                (token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode))
                            );
                        }
                    } else {
                        // (fixed... ugly but works) this can't handle non utf-8 chars... it just crashes because of the char-boundaries
                        let txt = tokenClone.get(0..token.text.len() - (
                                currentCharNum + token.text.len() - self.cursor.1
                        )).unwrap_or("").to_string();
                        let leftSize = txt.len();
                        if self.highlighting && (self.cursorEnd.0 < self.cursor.0 || self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 < self.cursor.1) {
                            if self.cursorEnd.1 > currentCharNum && self.cursor.1 <= currentCharNum + leftSize && self.cursorEnd.1 - currentCharNum < token.text.len() &&
                                self.cursor.0 == self.cursorEnd.0
                            {
                                coloredLeft.push((
                                    self.cursorEnd.1 - currentCharNum,  // this is greater than the text length.....
                                    self.GenerateColor(
                                        &token.token,
                                        txt[..self.cursorEnd.1 - currentCharNum].to_string(),
                                        colorMode
                                    )
                                ));
                                coloredLeft.push((
                                    txt.len() - (self.cursorEnd.1 - currentCharNum),
                                    self.GenerateColor(
                                        &token.token,
                                        txt[self.cursorEnd.1 - currentCharNum..].to_string(),
                                        colorMode
                                    ).on_dark_gray()
                                ));
                            } else {
                                coloredLeft.push((
                                    txt.len(),
                                    self.GenerateColor(&token.token, txt, colorMode).on_dark_gray()
                                ));
                            }
                        } else {
                            coloredLeft.push((
                                txt.len(),
                                self.GenerateColor(&token.token, txt, colorMode)
                            ));
                        }
                        if editingCode {
                            coloredLeft.push((1, "|"
                                .to_string()
                                .white()
                                .bold()))
                        };
                        let txt = tokenClone.get(
                            token.text.len() - (
                                currentCharNum + token.text.len() - self.cursor.1
                            )..token.text.len()
                        ).unwrap_or("").to_string();

                        if self.highlighting && (self.cursorEnd.0 > self.cursor.0 || self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 > self.cursor.1) {
                            if self.cursorEnd.1 > currentCharNum + leftSize && self.cursorEnd.1 < currentCharNum + token.text.len() {
                                coloredRight.push((
                                    self.cursorEnd.1 - (currentCharNum + leftSize),
                                    self.GenerateColor(
                                        &token.token,
                                        txt[..self.cursorEnd.1 - (currentCharNum + leftSize)].to_string(),
                                        colorMode
                                    ).on_dark_gray()
                                ));
                                coloredRight.push((
                                    txt.len() - (self.cursorEnd.1 - (currentCharNum + leftSize)),
                                    self.GenerateColor(
                                        &token.token,
                                        txt[self.cursorEnd.1 - (currentCharNum + leftSize)..].to_string(),
                                        colorMode
                                    )
                                ));
                            } else {
                                coloredRight.push((
                                    txt.len(),
                                    self.GenerateColor(&token.token, txt, colorMode).on_dark_gray()
                                ));
                            }
                        } else {
                            coloredRight.push((
                                txt.len(),
                                self.GenerateColor(&token.token, txt, colorMode)
                            ));
                        }
                    }
                } else if (self.cursorEnd.0 < self.cursor.0 || self.cursorEnd.0 == self.cursor.0 && self.cursorEnd.1 < self.cursor.1) && self.highlighting {
                    if (lineNumber > self.cursorEnd.0 && lineNumber < self.cursor.0) ||
                        (lineNumber == self.cursor.0 && lineNumber == self.cursorEnd.0 &&
                            currentCharNum >= self.cursorEnd.1 && currentCharNum + token.text.len() <= self.cursor.1)
                    {
                        coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode)
                            .on_dark_gray()));
                    } else if currentCharNum + token.text.len() > self.cursorEnd.1 && currentCharNum < self.cursorEnd.1 && lineNumber == self.cursorEnd.0 {   // can't be equal to cursor line
                        let txtRight = tokenClone[self.cursorEnd.1 - currentCharNum..].to_string();
                        let txtLeft = tokenClone[..self.cursorEnd.1 - currentCharNum].to_string();
                        coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, txtLeft, colorMode)));
                        coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, txtRight, colorMode)
                            .on_dark_gray()));
                    } else if (lineNumber == self.cursor.0 && currentCharNum + token.text.len() <= self.cursor.1 ||
                        lineNumber == self.cursorEnd.0 && currentCharNum >= self.cursorEnd.1) && self.cursor.0 != self.cursorEnd.0
                    {
                        coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode)
                            .on_dark_gray()));
                    } else {
                        coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode)));
                    }
                } else if self.highlighting {
                    if (lineNumber > self.cursor.0 && lineNumber < self.cursorEnd.0) ||
                        (lineNumber == self.cursorEnd.0 && lineNumber == self.cursor.0 &&
                            currentCharNum >= self.cursor.1 && currentCharNum + token.text.len() <= self.cursorEnd.1)
                    {
                        coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode)
                            .on_dark_gray()));
                    } else if currentCharNum + token.text.len() > self.cursorEnd.1 && currentCharNum <= self.cursorEnd.1 && lineNumber == self.cursorEnd.0 {   // can't be equal to cursor line

                        let txtRight = tokenClone[self.cursorEnd.1 - currentCharNum..].to_string();
                        let txtLeft = tokenClone[..self.cursorEnd.1 - currentCharNum].to_string();
                        coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, txtLeft, colorMode)
                            .on_dark_gray()));
                        coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, txtRight, colorMode)));
                    } else if (
                        lineNumber == self.cursorEnd.0 &&
                            currentCharNum <= self.cursorEnd.1 ||
                            lineNumber == self.cursor.0 &&
                                currentCharNum >= self.cursor.1) &&
                        self.cursorEnd.0 != self.cursor.0
                    {
                        coloredLeft.push(
                            (token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode)
                                .on_dark_gray())
                        );
                    } else {
                        coloredLeft.push(
                            (token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode))
                        );
                    }
                } else {
                    coloredLeft.push((token.text.len(), self.GenerateColor(&token.token, tokenCloneStr, colorMode)));
                    //coloredLeft.push((1, "|".white()))  // shows the tokens    todo (just to pin this line idk)
                }

                currentCharNum += token.text.len();
            }

            // !the loop finished so the read is being dropped
            // it's unfortunately a very long time to have the read
            // but there aren't really any good ways to do otherwise
            drop(lineTokensRead);

            if lineNumber == self.cursor.0 && currentCharNum <= self.cursor.1 && editingCode {
                coloredLeft.push((1, "|".to_string().white().bold()));
                // adding the suggested add-on
                if !suggested.is_empty() {
                    let mut tokens = vec!();
                    self.GetCurrentToken(&mut tokens);
                    if !tokens.is_empty() {
                        let selectedToken = tokens.remove(0);
                        let partialToken = suggested
                            .get(selectedToken.len()..)
                            .unwrap_or("")
                            .to_string();

                        coloredLeft.push((
                            suggested.len().saturating_sub(selectedToken.len()),
                            partialToken.white().dim().italic()
                        ));
                    }
                }
            }

            let mut charCount = 0usize;
            let mut finalColText: Vec<Span> = vec!();
            for (size, col) in coloredLeft {
                if charCount + size >= (area.width - 29 - 4) as usize { break; }
                if self.cursor.0 == lineNumber && editingCode {
                    finalColText.push(col.underlined());
                } else {
                    finalColText.push(col);
                }
                charCount += size;
            }
            for (size, col) in coloredRight {
                if charCount + size >= (area.width - 29 - 4) as usize { break; }
                if self.cursor.0 == lineNumber && editingCode {
                    finalColText.push(col.underlined());
                } else {
                    finalColText.push(col);
                }
                charCount += size;
            }
            let scrollPercent = f64::min(std::cmp::max(
                self.scrolled as isize + self.mouseScrolled, 0
            ) as f64 / self.lines.len() as f64 * (area.height as f64 - 10.0),
                                         area.height as f64 - 12.0
            ) as usize;
            
            if scrollPercent.saturating_sub(1) <= i && i <= scrollPercent + 1 ||
                self.pinedLines.contains(&lineNumber) {
                let rightPadding = (area.width - 29 - 4) as usize - charCount;
                for _ in 0..rightPadding {
                    finalColText.push(" ".to_string().white());
                }

                // the scroll bar
                if scrollPercent > 0 && i == scrollPercent - 1 {
                    finalColText.push("/\\".to_string().white());
                } else if i == scrollPercent {
                    finalColText.push("||".to_string().white());
                } else if i == scrollPercent + 1 {
                    finalColText.push("\\/".to_string().white());
                } else {
                    finalColText.push("  ".to_string().on_red());
                    for spanIndex in 0..finalColText.len() {
                        finalColText[spanIndex] = finalColText[spanIndex]
                            .clone()
                            .underlined();
                    }
                }
            }

            tabText.push(Line::from(
                finalColText
            ));

            i += 1;
        }

        tabText
    }
}

impl Default for CodeTab {
    fn default() -> Self {
         CodeTab{
             cursor: (0, 0),
             lines: vec![],
             lineTokens: std::sync::Arc::new(parking_lot::RwLock::new(vec![])),
             scopeJumps: std::sync::Arc::new(parking_lot::RwLock::new(vec![])),
             scopes: std::sync::Arc::new(parking_lot::RwLock::new(ScopeNode {
                 children: vec![],
                 name: "Root".to_string(),
                 start: 0,
                 end: 0,
             })),
             linearScopes: std::sync::Arc::new(parking_lot::RwLock::new(vec![])),
             scrolled: 0,
             mouseScrolled: 0,
             mouseScrolledFlt: 0.0,
             name: "Welcome.txt"
                 .to_string(),
             fileName: "".to_string(),
             cursorEnd: (0, 0),
             highlighting: false,
             pauseScroll: 0,
             searchIndex: 0,
             searchTerm: String::new(),
             changeBuffer: vec!(),
             redoneBuffer: vec!(),
             pinedLines: vec!(),
             outlineKeywords: std::sync::Arc::new(parking_lot::RwLock::new(vec!())),
             lineTokenFlags: std::sync::Arc::new(parking_lot::RwLock::new(vec!())),
             scopeGenerationHandles: vec!(),
        }
    }
}


#[derive(Debug)]
pub struct CodeTabs {
    pub tabFileNames: Vec <String>,
    pub tabs: Vec <CodeTab>,
    pub currentTab: usize,
    pub panes: Vec <usize>,  // todo! add this
}

impl CodeTabs {
    pub fn CheckScopeThreads (&mut self) {
        for tab in self.tabs.iter_mut() {
            tab.CheckScopeThreads();
        }
    }

    pub fn GetRelativeTabPosition (&self, positionX: u16, area: Rect, paddingLeft: u16) -> u16 {
        let total = self.panes.len() as u16 + 1;
        let tabSize = (area.width - paddingLeft) / total;

        // getting the error
        let error = (area.width as f64 - paddingLeft as f64) / (total as f64) - tabSize as f64;

        let tabNumber = std::cmp::min(
            (positionX.saturating_sub(paddingLeft)) / tabSize,
            self.panes.len() as u16  // no need to sub one bc/ the main tab isn't in the vector
        );
        // error = 0.5
        // offset 0, 1, 0, 1, 0, 1
        // 0.5*(tab+1)
        let offset = (error * (tabNumber+1) as f64) as u16;
        // println!("Offset: {}", offset);
        positionX.saturating_sub(paddingLeft)
            .saturating_sub((tabSize * tabNumber) as u16)
            .saturating_sub(tabNumber)  // no clue why this is needed tbh
            .saturating_sub(offset)
    }

    pub fn GetTab (&mut self, area: &Rect, paddingLeft: usize, positionX: usize, lastTab: &mut usize) -> &mut CodeTab {
        let tab = self.GetTabNumber(area, paddingLeft, positionX, lastTab);
        &mut self.tabs[tab]
    }

    pub fn GetTabNumber (&self, area: &Rect, paddingLeft: usize, positionX: usize, lastTab: &mut usize) -> usize {
        let total = self.panes.len() + 1;
        let tabSize = (area.width as usize - paddingLeft) / total;
        let tabNumber = std::cmp::min(
            (positionX - paddingLeft) / tabSize,
            self.panes.len()  // no need to sub one bc/ the main tab isn't in the vector
        );
        if tabNumber == 0 {
            self.currentTab.clone_into(lastTab);
            self.currentTab
        } else {
            tabNumber.clone_into(lastTab);
            self.panes[tabNumber - 1]
        }
    }

    pub fn GetTabSize (&self, area: &Rect, paddingLeft: usize) -> usize {
        let total = self.panes.len() + 1;
        (area.width as usize - paddingLeft) / total
    }

    pub fn GetScrolledText <'a> (&mut self,
                                 area: Rect,
                                 editingCode: bool,
                                 colorMode: &Colors::ColorMode,
                                 suggested: &'a String,
                                 tabIndex: usize,
    ) -> Vec <ratatui::text::Line> {
        self.tabs[tabIndex].GetScrolledText(area, editingCode, colorMode, suggested)
    }

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
            self.currentTab -= 1;  // there's a condition ensuring its 1 or greater

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
                        format!(" ({}) ", index + 1)
                            .to_string()
                            .light_yellow()
                            .bold()
                            .on_dark_gray()
                            .underlined()
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
                    lineTokens: std::sync::Arc::new(parking_lot::RwLock::new(vec![])),
                    scopeJumps: std::sync::Arc::new(parking_lot::RwLock::new(vec![])),
                    scopes: std::sync::Arc::new(parking_lot::RwLock::new(ScopeNode {
                        children: vec![],
                        name: "Root".to_string(),
                        start: 0,
                        end: 2,
                    })),
                    linearScopes: std::sync::Arc::new(parking_lot::RwLock::new(vec![
                        vec![0]
                    ])),
                    scrolled: 0,
                    mouseScrolled: 0,
                    mouseScrolledFlt: 0.0,
                    name: "main.rs".to_string(),
                    fileName: "".to_string(),
                    cursorEnd: (0, 0),
                    highlighting: false,
                    pauseScroll: 0,
                    searchIndex: 0,
                    searchTerm: String::new(),
                    changeBuffer: vec!(),
                    redoneBuffer: vec!(),
                    pinedLines: vec!(),
                    outlineKeywords: std::sync::Arc::new(parking_lot::RwLock::new(vec!())),
                    lineTokenFlags: std::sync::Arc::new(parking_lot::RwLock::new(vec!())),
                    scopeGenerationHandles: vec!(),
                }

            ],  // put a tab here or something idk
            currentTab: 0,
            panes: vec![],
        }
    }
}


