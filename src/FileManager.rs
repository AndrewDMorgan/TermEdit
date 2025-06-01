/*
//// the returned file data!!! (for debugging; the returned data from the file tree)
FilePathNode
{
    pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/", paths: [FilePathNode
        {
            pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/proc_macros", paths: [FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/proc_macros/src", paths: [], dirFiles: ["lib.rs"], allItems: [("lib.rs", File)], collapsed: true
            }
            ], dirFiles: ["Cargo.toml"], allItems: [("Cargo.toml", File), ("src", Directory)], collapsed: true
        },
        FilePathNode
        {
            pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/target", paths: [FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/target/release", paths: [FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNod
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                }],
                dirFiles: [], allItems: [(".fingerprint", Directory), ("incremental", Directory), ("examples", Directory), ("deps", Directory), ("build", Directory)], collapsed: true
            },
            FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/target/criterion", paths: [FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                }
                ], dirFiles: [], allItems: [("reports", Directory)], collapsed: true
            },
            FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/target/debug", paths: [FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                }
                ], dirFiles: [], allItems: [(".fingerprint", Directory), ("incremental", Directory), ("examples", Directory), ("deps", Directory), ("build", Directory)], collapsed: true
            }], dirFiles: [], allItems: [("release", Directory), ("criterion", Directory), ("debug", Directory)], collapsed: true
        }, FilePathNode
        {
            pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.git", paths: [FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.git/objects", paths: [FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                }],
                dirFiles: [], allItems: [("pack", Directory), ("info", Directory)], collapsed: true
            },
            FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.git/info", paths: [], dirFiles: [], allItems: [], collapsed: true
            },
            FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.git/hooks", paths: [], dirFiles: [], allItems: [], collapsed: true
            },
            FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.git/refs", paths: [FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                },
                FilePathNode
                {
                    pathName: "", paths: [], dirFiles: [], allItems: [], collapsed: true
                }],
                dirFiles: [], allItems: [("heads", Directory), ("tags", Directory)], collapsed: true
            }],
            dirFiles: [], allItems: [("objects", Directory), ("info", Directory), ("hooks", Directory), ("refs", Directory)], collapsed: true
        },
        FilePathNode
        {
            pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.vscode", paths: [], dirFiles: [], allItems: [], collapsed: true
        },
        FilePathNode
        {
            pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/assets", paths: [],
            dirFiles: ["nullSyntaxHighlighting.lua",
            "luaSyntaxHighlighting.lua", "rustSyntaxHighlighting.lua", "cppSyntaxHighlighting.lua",
            "pythonSyntaxHighlighting.lua"],
            allItems: [("nullSyntaxHighlighting.lua", File),
            ("luaSyntaxHighlighting.lua", File), ("rustSyntaxHighlighting.lua", File),
            ("cppSyntaxHighlighting.lua", File), ("pythonSyntaxHighlighting.lua", File)], collapsed: true
        },
        FilePathNode
        {
            pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.idea", paths: [FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.idea/codeStyles", paths: [], dirFiles: [], allItems: [], collapsed: true
            },
            FilePathNode
            {
                pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/.idea/dictionaries", paths: [], dirFiles: [], allItems: [], collapsed: true
            }],
            dirFiles: [], allItems: [("codeStyles", Directory), ("dictionaries", Directory)], collapsed: true
        },
        FilePathNode
        {
            pathName: "/Users/Andrew/Desktop/Programing/Rust/TermEdit/src", paths: [],
            dirFiles: ["StringPatternMatching.rs", "Tokens.rs", "eventHandler.rs",
            "Colors.rs", "TermRender.rs", "FileManager.rs", "main.rs", "CodeTabs.rs"],
            allItems: [("StringPatternMatching.rs", File), ("Tokens.rs", File), ("eventHandler.rs", File),
            ("Colors.rs", File), ("TermRender.rs", File), ("FileManager.rs", File), ("main.rs", File),
            ("CodeTabs.rs", File)], collapsed: true
        }],
        dirFiles: ["Cargo.toml"], allItems: [("Cargo.toml", File),
        ("proc_macros", Directory), ("target", Directory), (".git", Directory), (".vscode", Directory),
        ("assets", Directory), (".idea", Directory), ("src", Directory)], collapsed: true
}
*/

use crate::{CodeTabs, FileTabs};
use crate::Tokens::{GenerateTokens, ScopeNode};
use crate::TermRender::*;
use std::path::PathBuf;
use dirs::home_dir;
use std::io;
use proc_macros::color;
use crate::TermRender::{ColorType, Span};

use crate::{AppState, TabState};
use crate::App as MainApp;
use crate::CodeTabs::CodeTab;
use crate::eventHandler::{KeyCode, KeyParser, MouseEvent, MouseEventType, MouseState};

static VALID_EXTENSIONS: [&str; 10] = [
    "txt",
    "rs",
    "py",
    "cpp",
    "hpp",
    "c",
    "h",
    "lua",
    "toml",
    "json",
];

static MAX_FILE_EMBEDDING_DEPTH: usize = 4usize;


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
                    // this is a mess... (ya.......).....not even gonna try to figure this mess out
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
            for (index, itemsInfo) in self.allFiles.iter().enumerate() {
                fileText.push(MainApp::RenderFile(
                    app, index == self.fileBrowser.fileCursor, itemsInfo
                ));
            }
        }

        let window = app.GetWindowReferenceMut(String::from("Files"));
        let changed =
            if self.appState == AppState::CommandPrompt && self.tabState == TabState::Files {
                window.TryColorize(ColorType::BrightBlue)
            } else {
                window.ClearColors()
        };
        window.TryUpdateLines(fileText);

        if changed {  app.GetWindowReferenceMut(String::from("FileOptions")).UpdateAll();  }

        self.RenderFileOptions(app);
    }

    fn RenderFileOptions (&mut self, app: &mut App) {
        let mut filesText = color!["| Files |", Underline, White];
        if self.fileBrowser.fileOptions.selectedOptionsTab == OptionTabs::Files {
            filesText = color![filesText, OnBlue];
        }

        let window = app.GetWindowReferenceMut(String::from("FileOptions"));
        window.TryUpdateLines(vec![Span::FromTokens(vec![
            filesText,
        ])]);

        let window = app.GetWindowReferenceMut(String::from("FileOptionsDrop"));
        let hidden =
            if self.fileBrowser.fileOptions.selectedOptionsTab == OptionTabs::Files {
                window.Show();
                window.TryUpdateLines(vec![Span::FromTokens(vec![
                    color!["  ", White, OnBrightBlack],
                    color!["New File", BrightWhite, Underline, OnBrightBlack],
                    color!["   ", White, OnBrightBlack],
                ]), Span::FromTokens(vec![
                    color!["              ", White, OnBrightBlack],
                ]), Span::FromTokens(vec![
                    color!["              ", White, OnBrightBlack],
                ]), Span::FromTokens(vec![
                    color!["              ", White, OnBrightBlack],
                ]), Span::FromTokens(vec![
                    color!["              ", White, OnBrightBlack],
                ]), Span::FromTokens(vec![
                    color!["              ", White, OnBrightBlack],
                ])]);
                true
            } else {
                let hidden = window.hidden;
                window.Hide(); window.SupressUpdates();  // the file browser should cover it over
                hidden
        };

        if !hidden {
            let window = app.GetWindowReferenceMut(String::from("Files"));
            window.UpdateAll();  // making sure it doesn't get covered over
            if self.codeTabs.tabs.is_empty() {  return;  }
            // updating all code tabs
            for tabIndex in &self.codeTabs.panes {
                let name = self.codeTabs.tabs[*tabIndex].name.clone();
                app.GetWindowReferenceMut(format!("CodeBlock{name}")).UpdateAll();
            }
            let name = self.codeTabs.tabs[self.codeTabs.currentTab].name.clone();
            app.GetWindowReferenceMut(format!("CodeBlock{name}")).UpdateAll();
        }
    }

    pub fn RenderFile (app: &mut App, onCursor: bool, itemsInfo: &FileInfo) -> Span {
        Span::FromTokens(vec![
            {
                let padding = "  ".repeat(itemsInfo.depth);
                let dirSymbol =
                    if itemsInfo.fileType == FileType::Directory {
                        if itemsInfo.collapsed {  "> "  }
                        else {  "v "  }
                    } else {  ""  };
                if onCursor {
                    color![format!("{}{}{}", padding, dirSymbol, itemsInfo.name), BrightWhite, Underline]
                } else {
                    color![format!("{}{}{}", padding, dirSymbol, itemsInfo.name), BrightWhite]
                }
            }
        ])
    }

    fn HandlePressedOptions (&mut self, _events: &KeyParser, event: &MouseEvent) {
        // updating the windows
        match self.fileBrowser.fileOptions.selectedOptionsTab {
            OptionTabs::Files => {
                self.HandleFileOptionsPressed(event);
                return;
            }
            _ => {}
        }

        if event.position.1 == 1 {
            if event.position.0 >= 1 && event.position.0 < 9 {
                self.fileBrowser.fileOptions.selectedOptionsTab = OptionTabs::Files;
            } else {
                self.fileBrowser.fileOptions.selectedOptionsTab = OptionTabs::Null;
            }
        }
    }

    fn HandleFileOptionsPressed (&mut self, event: &MouseEvent) {
        if event.position.0 > 15 || event.position.1 <= 1 || event.position.1 > 30 {
            self.fileBrowser.fileOptions.selectedOptionsTab = OptionTabs::Null;
        }
    }

    fn HandleOptionsKeycodes (&mut self, events: &KeyParser) {
        if events.ContainsKeyCode(KeyCode::Escape) {
            self.fileBrowser.fileOptions.selectedOptionsTab = OptionTabs::Null;
        }
    }

    pub(crate) async fn PressedLoadFile (&mut self, events: &KeyParser) {
        if self.fileBrowser.fileOptions.selectedOptionsTab != OptionTabs::Null {
            self.HandleOptionsKeycodes(events);
        }

        if events.mouseEvent.is_none() {  return;  }
        let event = events.mouseEvent.as_ref().unwrap();

        // there aren't current any handlers that operate on other mouse events
        if event.eventType != MouseEventType::Left || event.state != MouseState::Press {  return;  }

        if event.position.1 <= 1 || self.fileBrowser.fileOptions.selectedOptionsTab != OptionTabs::Null {
            self.HandlePressedOptions(events, event);
            return;
        }

        let height = event.position.1.saturating_sub(2) as usize;
        let onFiles =
            event.position.0 < 30 && //height - 8, width 30
            event.position.1 > 1 &&
            self.appState == AppState::CommandPrompt &&
            self.allFiles.len() > height;
        // making sure it's not out of range
        if !onFiles {  return;  }

        self.HandleFilesMousePress(event, height).await;
    }

    async fn HandleFilesMousePress (&mut self, _event: &MouseEvent, height: usize) {
        // getting the file, and checking if it's a directory or not
        let fileInfo = &self.allFiles[height];
        if fileInfo.fileType == FileType::Directory {
            // opening the pathway
            let nthElement = self.fileBrowser.GetNthElement(height);
            if let Some(filePath) = nthElement {
                let file = self.fileBrowser.fileTree.GetLeaf(filePath);
                if let Some(file) = file {
                    file.collapsed = !file.collapsed;
                    self.RecalcAllFiles();
                }
            }

            return;  // no files need opening
        }
        // loading the file's contents
        self.codeTabs.currentTab = self.codeTabs.tabs.len();  // the next future element should be this file

        //let name = &self.fileBrowser.files[height];

        let mut lines: Vec <String> = vec!();

        //let fullPath = &self.fileBrowser.filePaths[height];

        let msg = fileInfo.path.as_str().trim();  // temporary for debugging (ya sure.... very temporary--5/30/25)
        let contents = std::fs::read_to_string(&fileInfo.path).expect(msg);
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
        tab.name = fileInfo.name.clone();

        tab.fileName = fileInfo.name.clone();
        tab.path = fileInfo.path.clone();

        tab.lineTokens.write().clear();
        let ending = tab.fileName.split('.').next_back().unwrap_or("");
        for (lineNumber, line) in tab.lines.iter().enumerate() {
            let value =
                GenerateTokens(line.clone(),
                               ending,
                               &tab.lineTokenFlags,
                               lineNumber,
                               &tab.outlineKeywords,
                               &self.luaSyntaxHighlightScripts
                ).await;
            tab.lineTokenFlags.write().push(vec!());
            tab.lineTokens.write().push(value);
        }
        tab.CreateScopeThread();
        //(tab.scopes, tab.scopeJumps, tab.linearScopes) = GenerateScopes(&tab.lineTokens, &tab.lineTokenFlags, &mut tab.outlineKeywords);

        self.codeTabs.tabs.push(tab);
        self.codeTabs.tabFileNames.push(fileInfo.name.clone());
    }

    /// Recalculates the cumulative file structure; if a directory is collapsed or expanded, this has to be recalculated.
    pub(crate) fn RecalcAllFiles (&mut self) {
        self.allFiles = self.fileBrowser.fileTree.CollectAllItems(0);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OptionTabs {
    #[default] Null,
    Files,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileOptionManager {
    pub selectedOptionsTab: OptionTabs,
}


// stores the information for the flattened files vector (there were wayyyy to many args....)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileInfo {
    pub name: String,
    pub fileType: FileType,
    pub depth: usize,
    pub collapsed: bool,
    pub path: String,
}


#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub enum FileType {
    #[default] File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct FilePathNode {
    pub pathName: String,
    pub paths: Vec <FilePathNode>,  // the following embedded paths
    pub dirFiles: Vec <String>,  // the files in the current directory
    pub allItems: Vec <(String, FileType)>,  // includes files and further directories
    pub collapsed: bool,
}

impl Default for FilePathNode {
    fn default() -> FilePathNode {
        FilePathNode {
            pathName: String::default(),
            paths: Vec::default(),
            dirFiles: Vec::default(),
            allItems: Vec::default(),
            collapsed: true,
        }
    }
}

impl FilePathNode {
    pub fn GetChild (&self, pathName: String) -> Option <FilePathNode> {
        for path in &self.paths {
            if path.pathName == pathName {
                return Some(path.clone());
            }
        } None
    }

    // given a set of node (aka files) names, this method will return the base leaf node
    pub fn GetLeaf (&mut self, mut pathNames: Vec <String>) -> Option <&mut FilePathNode> {
        let pathName = pathNames.remove(0);
        let mut dirCount = 0;
        for file in self.allItems.clone() {
            if file.1 == FileType::Directory {  dirCount += 1;  }
            if file.0 == pathName {
                if file.1 == FileType::Directory {
                    return
                        if !pathNames.is_empty() {  self.paths[dirCount - 1].GetLeaf(pathNames)  }
                        else {  Some(&mut self.paths[dirCount - 1])  };
                } return Some(self);
            }
        } None
    }

    pub fn CollectAllItems (&self, depth: usize) -> Vec <FileInfo> {
        let mut dirIndex = 0;
        // name?, fileType, depth, if it's collapsed or not (for rendering and stuff), file path
        let mut allFiles = vec![];
        for (file, fileType) in &self.allItems {
            if *fileType == FileType::Directory {
                allFiles.push(FileInfo {
                    name: file.clone(),
                    fileType: *fileType,
                    depth,
                    collapsed: self.paths[dirIndex].collapsed,
                    // I think the '/' seperator is needed (and not already in the path or name)
                    path: format!("{}/{}", self.pathName, file)  // the path to the file (not just the dir it's in)
                });
                if !self.paths[dirIndex].collapsed {
                    let mut embedded = self.paths[dirIndex].CollectAllItems(depth + 1);
                    allFiles.append(&mut embedded);
                }
                dirIndex += 1;
            } else {
                allFiles.push(FileInfo {
                    name: file.clone(),
                    fileType: *fileType,
                    depth,
                    collapsed: false,
                    // I think the '/' seperator is needed (and not already in the path or name)
                    path: format!("{}/{}", self.pathName, file)  // the path to the file (not just the dir it's in)
                });
            }
        }
        allFiles
    }
}

#[derive(Debug)]
pub struct FileBrowser {
    // this one would be the 0th element
    pub fileTree: FilePathNode,
    pub fileTab: FileTabs,
    pub fileCursor: usize,
    pub outlineCursor: usize,
    pub fileOptions: FileOptionManager,
}

impl Default for FileBrowser {
    fn default() -> FileBrowser {
        let mut node = FilePathNode::default();
        node.collapsed = false;
        FileBrowser {
            fileTree: node,
            fileTab: FileTabs::default(),
            fileCursor: usize::default(),
            outlineCursor: usize::default(),
            fileOptions: FileOptionManager::default(),
        }
    }
}

// manages the files for a given project
// provides an outline and means for loading files
impl FileBrowser {
    /// returns the file path to get to the file of the nth element (which could be a file or folder/branch/path)
    pub fn GetNthElement (&self, index: usize) -> Option <Vec <String>> {
        FileBrowser::SearchFiletree(&self.fileTree, &mut 0, index, 0)
    }

    fn SearchFiletree (path: &FilePathNode, i: &mut usize, index: usize, depth: usize) -> Option <Vec <String>> {
        if path.collapsed && depth > 0 {  return None;  }  // i shouldn't need to be incremented, right?
        let mut dirCount = 0;
        for (item, itemType) in &path.allItems {
            if *i == index {  return Some(vec![item.clone()]);  }
            *i += 1;
            if *itemType == FileType::Directory {
                let searchResults = FileBrowser::SearchFiletree(&path.paths[dirCount], i, index, depth + 1);
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
        // self.files.clear();
        self.fileTree = FilePathNode::default();
        self.fileTab = FileTabs::default();
        self.fileCursor = 0;
        self.outlineCursor = 0;
        codeTabs.tabs.clear();
        let pathInput = home_dir()
            .unwrap_or(PathBuf::from("/"))
            .join(indirectPathInput)
            .to_string_lossy()
            .into_owned();
        FileBrowser::LoadFilePathToTree(&mut self.fileTree, &pathInput, 0)
        //panic!("{:?}", self.fileTree);
        //Ok(())
    }

    pub fn LoadFilePathToTree (pathNode: &mut FilePathNode, pathInput: &str, depth: usize) -> io::Result <()> {
        if depth > MAX_FILE_EMBEDDING_DEPTH {  return Ok(());  }
        let paths = std::fs::read_dir(pathInput)?;
        pathNode.pathName = pathInput.to_owned();
        for pathResult in paths {
            let path = pathResult?;
            let metaData = path.file_type()?;
            if metaData.is_file() {
                let name = path.file_name().to_str().unwrap_or("").to_string();

                // so it doesn't try and load invalid files
                if !VALID_EXTENSIONS.contains(&name.split(".").last().unwrap_or("")) {  continue;  }

                pathNode.dirFiles.push(name.clone());
                pathNode.allItems.push((name, FileType::File));
                continue;
            } else if !metaData.is_dir() {  continue;  }

            let name = path.file_name().to_str().unwrap_or("").to_string();
            let newPath =
                if depth == 0 {  format!("{}{}", pathInput, name)  }
                else {  format!("{}/{}", pathInput, name)  };
            // generating the new path
            let mut newPathNode = FilePathNode::default();
            if name == *"src" {  newPathNode.collapsed = false;  }
            FileBrowser::LoadFilePathToTree(
                &mut newPathNode,
                &newPath,
                depth + 1
            )?;
            pathNode.paths.push(newPathNode);

            // adding the directory
            pathNode.allItems.push((name, FileType::Directory));
        } Ok(())
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

