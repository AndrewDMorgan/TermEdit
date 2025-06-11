use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

/// the number of checks before a complete refresh is called to
/// ensure proper synchronization of files (incase the differed
/// caching and call limiting is off/miss-aligned)
pub static SYNCHRONIZE_COUNT: usize = 600;  // about every minute (is this a good approach? hopefully it is)
// this should be less than the wait duration of within the runtime scheduler's soft shutdown sequence
static TIMEOUT_SAMPLE_FREQUENCY: usize = 4;  // about 40 milliseconds
static TIMEOUT: f64 = 15.0;  // in seconds

// possible events for the lsp to respond to
#[derive(Debug, Clone, PartialEq, Eq)]
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
    Lint,  // not sure if it needs any parameters? yes, it probably does?
    /// requests updates for the entire code file to ensure all changes are
    /// properly synchronized.
    Synchronize,
    /// the line, char_index to suggest completion options for
    Completion (usize, usize),
    /// goes to the definition based on the line, char_index of the event/action
    GotoDefinition (usize, usize),
}

// to interface between the code-tabs and the lsp
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustResponse {
    /// contains the types along with the range to update with them
    /// file, line start, line end, all types (this may need updating)
    UpdateTypeHints (String, usize, usize, Vec <String>),
    // other responses
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitStatus {
    /// a valid, non-problematic and uninterrupted polling
    Valid,
    /// the lsp took too long to respond (duration > TIMEOUT).
    /// Contains the event to allow a retry or other actions as deemed necessary
    /// by the caller.
    ResponseTimeOut (RustEvents),
}

type ResponseHandle = (std::thread::JoinHandle <()>,
                       crossbeam::channel::Sender <bool>,
                       crossbeam::channel::Receiver <String>
);
type Reader = std::sync::Arc <parking_lot::RwLock <BufReader <std::process::ChildStdout>>>;

#[derive(Debug, PartialEq, Eq)]
enum ThreadStatus {
    Busy,
    Idle
}

#[derive(Debug)]
pub struct RustAnalyzer {
    events: Vec <RustEvents>,
    stdin: std::process::ChildStdin,
    reader: Reader,

    responses: Vec <RustResponse>,
    responseHandler: ResponseHandle,
    backgroundThreadStatus: std::sync::Arc <parking_lot::RwLock <ThreadStatus>>,
}

impl Drop for RustAnalyzer {
    // seems to actually exit....
    fn drop(&mut self) {
        // making sure to safely exit regardless of what caused the drop (either exiting the app or crash)
        self.PromptLsp(r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#);
        let _ = self.GetResponse();
        // Now sending the exit notification
        self.PromptLsp(r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
    }
}

impl RustAnalyzer {
    fn PromptLsp (&mut self, message: &str) {
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", message.as_bytes().len(), message).unwrap();
        self.stdin.flush().unwrap();
    }

    /// pops a response (allows lsp responses to actually be acted upon by the proper code)
    pub fn PopResponse (&mut self) -> Option <RustResponse> {
        self.responses.pop()
    }

    /// this should only be used before or after core execution.
    fn GetResponse (&mut self) -> std::borrow::Cow <str> {
        let mut line = String::new();
        let mut contentLength = 0;
        loop {
            line.clear();
            self.reader.write().read_line(&mut line).unwrap();
            if line.trim().is_empty() {
                break;
            }
            if let Some(length) = line.strip_prefix("Content-Length:") {
                contentLength = length.trim().parse::<usize>().unwrap_or(0);
            }
        }
        let mut body = vec![0; contentLength];
        self.reader.write().read_exact(&mut body).unwrap();
        String::from_utf8_lossy(&body).into_owned().into()
    }

    /// initializes rust-analyzer (can only be called once; it is already called in RustAnalyzer::new())
    pub fn Initialize (&mut self) {
        // initializing the lsp
        self.PromptLsp(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params": {"capabilities": {},"rootUri": null}}"#);
        let _ = self.GetResponse();
        self.PromptLsp(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);
    }

    fn GetBackgroundThread (taskReceiver: crossbeam::channel::Receiver <bool>,
                            completionSender: crossbeam::channel::Sender <String>,
                            reader: Reader,
                            status: std::sync::Arc <parking_lot::RwLock <ThreadStatus>>,
    ) -> std::thread::JoinHandle<()> {
        // the os can clean the thread up after execution
        std::thread::spawn(move || {
            let Poll = || -> String {
                let mut line = String::new();
                let mut contentLength = 0;
                loop {
                    line.clear();
                    reader.write().read_line(&mut line).unwrap();
                    if line.trim().is_empty() {  break;  }
                    if let Some(length) = line.strip_prefix("Content-Length:") {
                        contentLength = length.trim().parse::<usize>().unwrap_or(0);
                    }
                }
                let mut body = vec![0; contentLength];
                reader.write().read_exact(&mut body).unwrap();
                String::from_utf8_lossy(&body).to_string()
            };

            loop {
                // polling
                std::thread::sleep(std::time::Duration::from_millis(25));
                if taskReceiver.try_recv().is_err() {  continue;  }
                *status.write() = ThreadStatus::Busy;
                let response = Poll();
                let _ = completionSender.send(response);
                *status.write() = ThreadStatus::Idle;
            }
        })
    }

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
            let reader = std::sync::Arc::new(parking_lot::RwLock::new(BufReader::new(stdout)));

            let status = std::sync::Arc::new(parking_lot::RwLock::new(ThreadStatus::Idle));
            let (taskSender, taskReceiver) = crossbeam::channel::bounded(1);
            let (completionSender, completionReceiver) = crossbeam::channel::bounded(1);
            let backgroundThread = Self::GetBackgroundThread(taskReceiver, completionSender, reader.clone(), status.clone());

            let mut instance = RustAnalyzer {
                events: vec![],
                stdin,
                reader,
                responses: Vec::new(),
                responseHandler: (backgroundThread, taskSender, completionReceiver),
                backgroundThreadStatus: status,
            };
            instance.Initialize();
            Some (instance)
        } else {  None  }
    }

    // gets the request to send to the lsp
    fn GetRequest (event: &RustEvents) -> () {
        match &event {
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
            RustEvents::Completion (line, charIndex) => {
                // todo!
            },
            RustEvents::GotoDefinition (line, charIndex) => {
                // todo!
            },
        }
    }

    // parses the response
    fn ParseResponse (event: RustEvents, response: String) -> Option <RustResponse> {
        match event {
            RustEvents::Workspace (filePath) => {
                // todo!
                None
            },
            RustEvents::UpdatedLines (line, charIndex, suggestCompletion) => {
                // todo!
                None
            },
            RustEvents::Hover (line, charIndex) => {
                // todo!
                None
            },
            RustEvents::Lint => {
                // todo!
                None
            },
            RustEvents::Synchronize => {
                // todo!
                None
            },
            RustEvents::Completion (line, charIndex) => {
                // todo!
                None
            },
            RustEvents::GotoDefinition (line, charIndex) => {
                // todo!
                None
            },
        }
    }

    // listens for a response from the lsp
    async fn ListenForResponse (&mut self) -> Option <String> {
        // waiting for the background thread to be free
        // this shouldn't ever be an issue unless there was a timeout
        // this is non-blocking which ensures a safe exit is still possible
        // the listener thread doesn't require a soft exit so blocking tasks there are safe
        loop {
            // waiting for the thread to be idle
            if *self.backgroundThreadStatus.read() == ThreadStatus::Idle {
                break;
            }
            futures::pending!();
            // in this context this is safe because of the custom runtime manager/scheduler
            std::thread::sleep(std::time::Duration::from_millis(250));
        }

        // requesting the background thread to begin gathering the response
        self.responseHandler.1.send(true).unwrap();

        // listening for and responding to a response
        let response;
        let mut iterationCount = 0;
        let start = std::time::Instant::now();
        loop {
            // polling for a response
            if let Ok(responseGathered) = self.responseHandler.2.try_recv() {
                response = responseGathered;
                break;
            }

            iterationCount += 1;
            // only checking the time so often to prevent excessive cpu usage
            if iterationCount > TIMEOUT_SAMPLE_FREQUENCY {
                // yielding to the executor to ensure a proper exit sequence is always possible
                // but not too often such that it creates a bottleneck
                futures::pending!();
                iterationCount = 0;
                if std::time::Instant::now().duration_since(start).as_secs_f64() > TIMEOUT {
                    // the background thread will continue to process (it's status is busy until it finishes)
                    // if the lsp got completely stuck, it's likely it won't be fixed
                    return None;  // incase the lsp fails to respond (preventing a complete lockup)
                }
            }

            // waiting to reduce cpu usage
            // the blocking context is safe while using the custom runtime
            std::thread::sleep(std::time::Duration::from_millis(10));
        } Some(response)
    }
    
    // this is async just incase it's needed (ensures it can function so no future work would be necessary)
    // updating anything related to the lsp
    pub async fn Poll (&mut self) -> ExitStatus {
        // acting on any requested events
        while let Some(event) = self.events.pop() {
            // sending the request
            let request = Self::GetRequest(&event);
            // todo! -- actually send the request.....
            
            // waits for a response unless there's a timeout
            // this occasionally yields to the executor giving it a chance to check its exit status
            let response = self.ListenForResponse().await;
            if response.is_none() {  return ExitStatus::ResponseTimeOut(event);  }  // checking if a time-out occurred

            // handling/parsing the response
            let response = Self::ParseResponse(event, response.unwrap());
            // adding the response
            if let Some(response) = response {
                self.responses.push(response);
            }
        } ExitStatus::Valid
    }

    /// Adds a new event to be handled when this instance is polled.
    pub fn NewEvent (&mut self, event: RustEvents) {
        self.events.push(event);
    }

    /// Adds a new event, except it goes to the back of the queue
    pub fn NewEventBack (&mut self, event: RustEvents) {
        self.events.push(event);
    }
}

