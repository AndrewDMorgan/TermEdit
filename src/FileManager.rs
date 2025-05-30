use crate::{CodeTabs, FileTabs};
use crate::Tokens::ScopeNode;
use crate::TermRender::*;
use std::path::PathBuf;
use dirs::home_dir;
use std::io;
use proc_macros::color;
use crate::TermRender::{ColorType, Span};

use crate::{AppState, TabState};
use crate::App as MainApp;

// implementing the main rendering logic for the filebrowser and scopes in this file
impl <'a> MainApp <'a> {
    fn RenderOutlinePartOne (&self, scopeIndex: &Vec <usize>) -> String {
        let mut offset = String::new();
        if *scopeIndex ==
            self.codeTabs.tabs[self.lastTab]
                .scopeJumps.read()[self.codeTabs.tabs[self.lastTab].cursor.0] &&
            self.appState == AppState::Tabs && self.tabState == TabState::Code {
            offset.push('>')
        } else if
        self.appState == AppState::CommandPrompt &&
            self.tabState == TabState::Files &&
            self.codeTabs.tabs[self.lastTab].linearScopes.read()[
                self.fileBrowser.outlineCursor
                ] == *scopeIndex {
            offset.push('>');
        }
        for _ in 0..scopeIndex.len().saturating_sub(1) {
            offset.push_str("  ");
        }
        offset
    }

    fn GetColoredScope (&self, scopeName: String, scopeLength: usize) -> Colored {
        match scopeLength {
            1 => color![scopeName, BrightBlue],
            2 => color![scopeName, BrightMagenta],
            3 => color![scopeName, BrightRed],
            4 => color![scopeName, BrightYellow],
            5 => color![scopeName, BrightGreen],
            _ => color![scopeName, BrightWhite],
        }
    }

    fn GetFilebrowserOutline (&self, fileStringText: &mut Vec <Span>, scopeIndex: &Vec <usize>, scope: &std::sync::Arc <parking_lot::RwLock<ScopeNode>>) {
        fileStringText.push(
            Span::FromTokens(vec![
                {
                    color![self.RenderOutlinePartOne(scopeIndex), BrightWhite]
                },
                {
                    // this is a mess... (ya.......)
                    if *scopeIndex ==
                        self.codeTabs.tabs[self.lastTab]
                            .scopeJumps.read()[self.codeTabs.tabs[self.lastTab].cursor.0] &&
                        self.appState == AppState::CommandPrompt &&
                        self.tabState == TabState::Code || (
                        self.appState == AppState::CommandPrompt &&
                            self.tabState == TabState::Files &&
                            self.codeTabs.tabs[self.lastTab].linearScopes.read()[
                                self.fileBrowser.outlineCursor
                                ] == *scopeIndex
                    )
                    {
                        color![self.GetColoredScope(scope.read().name.clone(), scopeIndex.len()), Underline]
                    } else {
                        color![self.GetColoredScope(scope.read().name.clone(), scopeIndex.len())]
                    }
                },
                //format!(" ({}, {})", scope.start + 1, scope.end + 1).white(),  // (not enough space for it to fit...)
            ])
        );
    }

    fn HandleScrolled(&self, scrolled: usize, newScroll: &mut usize, scopeIndex: &Vec <usize>) {
        let tab = &self.codeTabs.tabs[self.lastTab];
        if *scopeIndex == tab.scopeJumps.read()[tab.cursor.0] &&
            self.appState == AppState::Tabs && self.tabState == TabState::Code
        {
            *newScroll = scrolled - 1;
        }
    }

    fn RenderFilebrowserOutline (&mut self, area: &Rect) -> Vec <Span> {
        let mut fileStringText = vec!();
        let mut scopes: Vec<usize> = vec![];

        let mut newScroll = self.fileBrowser.outlineCursor;
        let mut scrolled = 0;
        let scrollTo = self.fileBrowser.outlineCursor.saturating_sub(((area.height - 8) / 2) as usize);

        let jumps = self.codeTabs.tabs[self.lastTab].scopeJumps.read();
        for scopeIndex in jumps.iter() {
            self.RenderInnerFilebrowserRender(&mut scopes, scopeIndex, &mut scrolled, scrollTo, &mut newScroll, &mut fileStringText);
        }
        self.fileBrowser.outlineCursor = newScroll;
        fileStringText
    }

    fn RenderInnerFilebrowserRender (&self, scopes: &mut Vec <usize>, scopeIndex: &Vec <usize>, scrolled: &mut usize, scrollTo: usize, newScroll: &mut usize, fileStringText: &mut Vec <Span>) {
        let mut valid = true;
        for i in 0..scopes.len() {
            let slice = scopes.get(0..(scopes.len() - i));
            if slice.unwrap_or(&[]) != *scopeIndex {  continue;  }
            valid = false;
            break;
        }
        if !valid || scopeIndex.is_empty() {  return;  }
        scopes.clear();

        {
            let scopesWrite = &mut self.codeTabs.tabs[self.lastTab].scopes.write();
            for index in scopeIndex {
                scopes.push(*index);
                if *index >= scopesWrite.children.len() {  continue;  }
                **scopesWrite = scopesWrite.children[*index].clone();
            }  // the write is naturally dropped
        }

        *scrolled += 1;
        self.HandleScrolled(*scrolled, newScroll, scopeIndex);

        if *scrolled < scrollTo { return; }
        self.GetFilebrowserOutline(fileStringText, scopeIndex, &self.codeTabs.tabs[self.lastTab].scopes);
    }

    // ============================================= files =============================================
    pub(crate) fn RenderFiles (&mut self, app: &mut App) {
        let mut fileText = vec![];

        if self.fileBrowser.fileTab == FileTabs::Outline {
            fileText = self.RenderFilebrowserOutline(app.GetWindowArea());
        } else {
            //let mut allFiles = vec!();
            for (index, file) in self.fileBrowser.files.iter().enumerate() {
                fileText.push(Span::FromTokens(vec![
                    {
                        if index == self.fileBrowser.fileCursor {
                            color![file, BrightWhite, Underline]
                        } else {
                            color![file, BrightWhite]
                        }
                    }
                ]));
            }
            //fileText = Text::from(allFiles);
        }

        {
            let window = app.GetWindowReferenceMut(String::from("Files"));

            if self.appState == AppState::CommandPrompt && self.tabState == TabState::Files {
                window.TryColorize(ColorType::BrightBlue);
            } else {
                window.ClearColors();
            }

            window.TryUpdateLines(fileText);
        }
    }
}


#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum FileType {
    #[default] File,
    Directory,
}

#[derive(Debug, Default, Clone)]
pub struct FilePathNode {
    pub pathName: String,
    pub paths: Vec <FilePathNode>,  // the following embedded paths
    pub dirFiles: Vec <String>,  // the files in the current directory
    pub allItems: Vec <(String, FileType)>,  // includes files and further directories
}

impl FilePathNode {
    pub fn GetChild (&self, pathName: String) -> Option <FilePathNode> {
        for path in &self.paths {
            if path.pathName == pathName {
                return Some(path.clone());
            }
        } None
    }

    pub fn GetLeaf (&self, mut pathNames: Vec <String>) -> Option <FilePathNode> {
        let pathName = pathNames.pop().unwrap_or_default();
        if self.paths.is_empty() {
            for file in &self.dirFiles {
                if *file == pathName {
                    return Some(self.clone());
                }
            }
        }
        for path in &self.paths {
            if path.pathName == pathName {
                return path.GetLeaf(pathNames);
            }
        } None
    }
}

#[derive(Debug, Default)]
pub struct FileBrowser {
    pub files: Vec <String>,  // stores the names
    pub filePaths: Vec <String>,  // these two are here until the rest of the code is updated (temporary, to allow it to function)

    // this one would be the 0th element
    pub fileTree: FilePathNode,
    pub fileTab: FileTabs,
    pub fileCursor: usize,
    pub outlineCursor: usize,
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
    /// returns the file path to get to the file of the nth element (which could be a file or folder/branch/path)
    pub fn GetNthElement (&self, index: usize) -> Option <Vec <String>> {
        FileBrowser::SearchFiletree(&self.fileTree, &mut 0, index)
    }

    fn SearchFiletree (path: &FilePathNode, i: &mut usize, index: usize) -> Option <Vec <String>> {
        let mut dirCount = 0;
        for (item, itemType) in &path.allItems {
            if *i == index {  return Some(vec![item.clone()]);  }
            *i += 1;
            if *itemType == FileType::Directory {
                let searchResults = FileBrowser::SearchFiletree(&path.paths[dirCount], i, index);
                if searchResults.is_some() {
                    let mut output = searchResults.unwrap();
                    output.insert(0, item.clone());
                    return Some(output);
                }
                dirCount += 1;
            }
        } None
    }

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
                if std::fs::FileType::is_file(&path.file_type()?) {
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
        if self.fileTab == FileTabs::Outline {
            self.outlineCursor = std::cmp::min(
                self.outlineCursor + 1,
                outline.len() - 1
            );
        } else {
            // todo
        }
    }

    pub fn MoveCursorUp (&mut self) {
        if self.fileTab == FileTabs::Outline {
            self.outlineCursor = self.outlineCursor.saturating_sub(1);  // simple
        } else {
            // todo
        }
    }
}

