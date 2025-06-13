use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use crossterm::tty::IsTty;

/// the number of checks before a complete refresh is called to
/// ensure proper synchronization of files (incase the differed
/// caching and call limiting is off/miss-aligned)
pub static SYNCHRONIZE_COUNT: usize = 600;  // about every minute (is this a good approach? hopefully it is)
// this should be less than the wait duration of within the runtime scheduler's soft shutdown sequence
static TIMEOUT_SAMPLE_FREQUENCY: usize = 4;  // about 40 milliseconds
static TIMEOUT: f64 = 15.0;  // in seconds
static RUST_ANALYZER_PATH: &str = "rust-analyzer";
static MAX_IDLE_CHECKS: usize = 20;  // 50 ms per check * 20 = 1 second

// possible events for the lsp to respond to
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustEvents {
    /// informs the lsp of a change in workspace (opening/closing a given project)
    Workspace (String),
    /// informs the lsp of an update in a range of given lines (used for inlaid type hints)
    /// this will also call for code completion suggestions
    /// (line number, end line, char index, completion?)
    UpdatedLines (usize, usize, usize, String),
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
    OpenedFile (String),
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
    /// an error was thrown somewhere in the process of gathering the response
    Error,
    /// when the cycle times out due to the reading thread being blocked by a long-running item
    ReadingBlockedTimeOut,
}

/// the background thread's join handle, channel to prompt it to start polling, and the results from polling
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
    pub filePath: (String, std::sync::Arc <parking_lot::RwLock <String>>),
    dropped: bool,
}

impl Drop for RustAnalyzer {
    // seems to actually exit....
    fn drop(&mut self) {
        if !self.dropped {
            self.DropConnection();
        }
    }
}

impl RustAnalyzer {
    /// drops the connection (used in .drop)
    pub fn DropConnection (&mut self) {
        self.dropped = true;
        // waiting till x time or till the LSPs status is no longer busy
        // this is just to make sure the lsp isn't current busy doing work
        // this would be called after the executor called .join so blocking tasks are safe
        let start = std::time::Instant::now();
        while std::time::Instant::now().duration_since(start).as_secs_f64() < 1.5 {
            if *self.backgroundThreadStatus.read() == ThreadStatus::Idle {
                break;
            } std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // making sure to safely exit regardless of what caused the drop (either exiting the app or crash)
        let _ = self.PromptLsp(r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#);
        let _ = self.GetResponse();
        // Now sending the exit notification
        let _ = self.PromptLsp(r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
    }

    /// Prompts the LSP with a given message -- ik, crazy, who could've guessed?
    /// Throws an error if stdin is invalid (or another fault within write! or flush).
    fn PromptLsp (&mut self, message: &str) -> Result <(), std::io::Error> {
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", message.as_bytes().len(), message)?;
        self.stdin.flush()?;
        Ok(())
    }

    /// pops a response (allows lsp responses to actually be acted upon by the proper code)
    pub fn PopResponse (&mut self) -> Option <RustResponse> {  self.responses.pop()  }

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
    pub fn Initialize (&mut self, filePath: String) -> Result <(), std::io::Error> {
        // initializing the lsp
        let msg = format!(
            "{}{}{}", r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params": {"capabilities": {},"rootUri": "file://"#,
            filePath, r#""}}"#
        );
        self.PromptLsp(&msg)?;
        let _ = self.GetResponse();
        self.PromptLsp(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)?;
        Ok(())
    }

    fn GetBackgroundThread (taskReceiver: crossbeam::channel::Receiver <bool>,
                            completionSender: crossbeam::channel::Sender <String>,
                            reader: Reader,
                            status: std::sync::Arc <parking_lot::RwLock <ThreadStatus>>,
    ) {
        // waiting for the single to begin
        let _ = taskReceiver.recv();

        // all of this runs on the background thread
        loop {
            // polling
            std::thread::sleep(std::time::Duration::from_millis(25));

            let mut line = String::with_capacity(256);
            let mut contentLength = 0;
            loop {
                line.clear();
                reader.write().read_line(&mut line).unwrap();
                *status.write() = ThreadStatus::Busy;  // once data starts to flow this should change
                if line.trim().is_empty() {  break;  }
                if let Some(length) = line.strip_prefix("Content-Length:") {
                    contentLength = length.trim().parse::<usize>().unwrap_or(0);
                }
            }
            *status.write() = ThreadStatus::Idle;
            let mut body = vec![0; contentLength];
            reader.write().read_exact(&mut body).unwrap();
            let response = String::from_utf8_lossy(&body).to_string();

            // times out after 1.5 seconds (incase a message got left in the channel)
            let _ = completionSender.send_timeout(response, std::time::Duration::from_millis(1500));
            *status.write() = ThreadStatus::Idle;
        }
    }

    pub fn new (filePath: String) -> Option <Self> {
        // https://rust-analyzer.github.io/book/rust_analyzer_binary.html
        let child = Command::new(RUST_ANALYZER_PATH)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())  // to hopefully prevent the terminal from getting spammed
            .spawn();
        if child.is_err() {  return None;  }
        let mut child = child.unwrap();

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        if stdin.is_none() || stdout.is_none() {  return None;  }
        let stdin = stdin.unwrap();
        let stdout = stdout.unwrap();
        let reader = std::sync::Arc::new(parking_lot::RwLock::new(BufReader::new(stdout)));

        let status = std::sync::Arc::new(parking_lot::RwLock::new(ThreadStatus::Idle));
        let (taskSender, taskReceiver) = crossbeam::channel::bounded(1);
        let (completionSender, completionReceiver) = crossbeam::channel::bounded(1);
        let function = Self::GetBackgroundThread;
        // the os can clean the thread up after execution
        let readerClone = reader.clone();
        let statusClone = status.clone();
        let backgroundThread = std::thread::spawn(move || {
            function(taskReceiver, completionSender, readerClone, statusClone);
        });

        let filePathSendSync = std::sync::Arc::new(parking_lot::RwLock::new(filePath.clone()));
        let mut instance = RustAnalyzer {
            events: vec![],
            stdin,
            reader,
            responses: Vec::new(),
            responseHandler: (backgroundThread, taskSender, completionReceiver),
            backgroundThreadStatus: status,
            filePath: (String::new(), filePathSendSync),
            dropped: false,
        };
        let status = instance.Initialize(filePath);
        if status.is_err() {  return None;  }  // something went wrong somewhere
        let _ = instance.responseHandler.1.send(true);  // telling the background thread to start
        Some (instance)
    }

    // gets the request to send to the lsp
    fn GetRequest<'a> (&mut self, event: &RustEvents) -> String {
        // contains the parameters and method names I think
        // https://rust-analyzer.github.io/book/contributing/lsp-extensions.html#configuration-in-initializationoptions
        match event {
            RustEvents::Workspace (pathway) => {
                // todo!   (this one shouldn't be used?)
                String::from("")
            },
            RustEvents::UpdatedLines (start, end, charIndex, text) => {
                // todo!
                let left =
                    r#"{
                        "textDocument": {
                            "uri": "file://"#;
                let right = r#"",
                            "version": 2
                        }
                    }"#;
                format!("{}{}{}", left, self.filePath.1.read(), right)
            },
            RustEvents::Hover (line, charIndex) => {
                // todo!
                String::from("")
            },
            RustEvents::Lint => {
                // todo!
                String::from("")
            },
            RustEvents::Synchronize => {
                // todo!
                String::from("")
            },
            RustEvents::Completion (line, charIndex) => {
                // todo!
                String::from("")
            },
            RustEvents::GotoDefinition (line, charIndex) => {
                // todo!
                String::from("")
            },
            RustEvents::OpenedFile (path) => {
                // todo!
                String::from("")
            },
        }
    }

    // parses the response
    fn ParseResponse (event: &RustEvents, response: String, filePath: &str) -> Option <RustResponse> {
        // first check if the response is a diagnostic instead of an event response
        // if it's a diagnostic process independent of the event

        // parsing the event
        match event {
            RustEvents::Workspace (filePath) => {
                // todo!
                None
            },
            RustEvents::UpdatedLines (start, end, charIndex, text) => {
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
            RustEvents::OpenedFile (path) => {
                // todo!
                None
            },
        }
    }

    // listens for a response from the lsp
    async fn ListenForResponse (&mut self) -> Result <Vec <String>, ExitStatus> {
        let mut responses = vec![];

        // going through all the existing responses to clear them out
        loop {
            self.WaitForIdle().await?;
            if let Ok(response) = self.responseHandler.2.try_recv() {
                responses.push(response);
            } else {
                break;
            } futures::pending!();  // making sure it doesn't cause weird blocking
            std::thread::sleep(std::time::Duration::from_millis(75));  // making sure other events have a second to jump in
        }

        // requesting the background thread to begin gathering the response
        //self.responseHandler.1.send(true).unwrap(); deprecated -- no longer needed

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

            // !===============!  Handling Timeouts  !===============!
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
                    return Ok(responses);  // incase the lsp fails to respond (preventing a complete lockup)
                }
            }

            // waiting to reduce cpu usage
            // the blocking context is safe while using the custom runtime
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        responses.push(response);
        Ok(responses)
    }

    async fn WaitForIdle (&mut self) -> Result <(), ExitStatus> {
        // max iteration count to allow an easy timeout
        for _ in 0..MAX_IDLE_CHECKS {  // waiting to ensure no tasks are overwritten
            // waiting for the thread to be idle
            if *self.backgroundThreadStatus.read() == ThreadStatus::Idle {
                return Ok(());
            }
            futures::pending!();
            // in this context this is safe because of the custom runtime manager/scheduler
            // this also shouldn't really ever run that often unless something gets timed out
            // this should always be less than the timeout duration within the executor's soft-shutdown
            std::thread::sleep(std::time::Duration::from_millis(50));
        } Err(ExitStatus::ReadingBlockedTimeOut)
    }
    
    // this is async just incase it's needed (ensures it can function so no future work would be necessary)
    // updating anything related to the lsp
    pub async fn Poll (&mut self) -> ExitStatus {
        // getting a clone to ensure the file won't change mid-way through
        // any current events should have taken place on this current file
        // hopefully this will prevent all but the most subtle and rare rare conditions
        let filePath = format!("{}/{}", self.filePath.0, self.filePath.1.read());

        // clearing out any existing events

        // acting on any requested events
        while let Some(event) = self.events.pop() {
            match self.WaitForIdle().await {
                Err(exitStatus) => return exitStatus,
                Ok(_) => {},
            }

            // sending the request
            let request = self.GetRequest(&event);
            //println!("Request: <    :{}:    >", request);
            //let status = self.PromptLsp(&request);  // sending the request
            //if status.is_err() {  return ExitStatus::Error;  }

            // waits for a response unless there's a timeout
            // this occasionally yields to the executor giving it a chance to check its exit status
            let responses = match self.ListenForResponse().await {
                Ok(responses) => responses,
                Err(exitStatus) => return exitStatus
            };
            if responses.is_empty() {  return ExitStatus::ResponseTimeOut(event);  }  // checking if a time-out occurred

            // handling/parsing the responses
            for response in responses {
                //println!("Response: <    :{}:    >", response);
                let response = Self::ParseResponse(&event, response, &filePath);
                // adding the response
                if let Some(response) = response {
                    self.responses.push(response);
                }
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

