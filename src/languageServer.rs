use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

/// the number of checks before a complete refresh is called to
/// ensure proper synchronization of files (incase the differed
/// caching and call limiting is off)
pub static SYNCHRONIZE_COUNT: usize = 600;  // about every minute

// possible events for the lsp to respond to
#[derive(Debug, Clone)]
pub enum RustEvents {
    /// informs the lsp of a change in workspace (opening/closing a given project)
    Workspace (String),
    /// informs the lsp of an update in a range of given lines (used for inlaid type hints)
    /// this will also call for code completion suggestions
    /// (line number, character number, completion?)
    UpdatedLines (usize, usize, bool),
    /// the (line number, character number) of the mouse
    Hover (usize, usize),
    /// lints the whole program to diagnostics (this should only be done after the user
    /// has stopped typing for some amount of time).
    Lint,  // not sure if it needs any parameters?
    //GoToDefinition: (),  // not sure what parameter it'll need todo!
    /// requests updates for the entire code file to ensure all changes are
    /// properly synchronized.
    Synchronize,
}

#[derive(Debug)]
pub struct RustAnalyzer {
    events: Vec <RustEvents>,
    stdin: std::process::ChildStdin,
    reader: BufReader <std::process::ChildStdout>,
}

impl Drop for RustAnalyzer {
    // seems to actually exit....
    fn drop(&mut self) {
        // making sure to safely exit regardless of what caused the drop (either exiting the app or crash)
        let shutdown_msg = r#"{"jsonrpc":"2.0","id":1,"method":"shutdown","params":null}"#;
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", shutdown_msg.len(), shutdown_msg).unwrap();
        self.stdin.flush().unwrap();

        // Wait for the shutdown response
        let mut line = String::new();
        let mut content_length = 0;

        // Parse headers to get Content-Length
        loop {
            line.clear();
            self.reader.read_line(&mut line).unwrap();
            if line.trim().is_empty() {
                break;
            }
            if let Some(length) = line.strip_prefix("Content-Length:") {
                content_length = length.trim().parse::<usize>().unwrap_or(0);
            }
        }

        // Read the response body
        let mut body = vec![0; content_length];
        self.reader.read_exact(&mut body).unwrap();

        // Optionally, validate it contains `"id":1`
        let response_str = String::from_utf8_lossy(&body);
        if !response_str.contains(r#""id":1"#) {
            eprintln!("Unexpected shutdown response: {}", response_str);
        }

        // Now send the exit notification
        let exit_msg = r#"{"jsonrpc":"2.0","method":"exit","params":null}"#;
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", exit_msg.len(), exit_msg).unwrap();
        self.stdin.flush().unwrap();
    }
}

impl RustAnalyzer {
    pub fn new() -> Option <Self> {
        // https://rust-analyzer.github.io/book/rust_analyzer_binary.html
        if let Ok(mut child) = Command::new("rust-analyzer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())  // to hopefully prevent the terminal from getting spammed
            .spawn()
        {
            let stdin = child.stdin.take();
            let stdout = child.stdout.take();
            if stdin.is_none() || stdout.is_none() {  return None;  }
            let stdin = stdin.unwrap();
            let stdout = stdout.unwrap();
            let reader = BufReader::new(stdout);
            Some (RustAnalyzer {
                events: vec![],
                stdin,
                reader,
            })
        } else {  None  }
    }

    // this is async just incase it's needed (ensures it can function so no future work would be necessary)
    // updating anything related to the lsp
    pub async fn Poll (&mut self) {
        // acting on any requested events
        while let Some(event) = self.events.pop() {
            // sending the request
            let request = match event {
                RustEvents::Workspace (filePath) => {
                    // todo!
                },
                RustEvents::UpdatedLines (line, charIndex, suggestCompletion) => {
                    // todo!
                },
                RustEvents::Hover (line, charIndex) => {
                    // todo!
                },
                RustEvents::Lint => {
                    // todo!
                },
                RustEvents::Synchronize => {
                    // todo!
                },
            };

            // awaiting the response (blocking tasks are safe here)
            // todo!
        }
    }

    /// Adds a new event to be handled when this instance is polled.
    pub fn NewEvent (&mut self, event: RustEvents) {
        self.events.push(event);
    }
}

