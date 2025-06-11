use std::task::{Context, Poll};
use std::pin::Pin;

/// Manages a pool of runtimes that can handle futures from synchronous and/or asynchronous code.
#[derive(Debug, Default)]
pub struct Runtime {
    // sender is for indicating a required exit; receiver is for checking if completed
    threadPool: Vec <(std::thread::JoinHandle <()>, crossbeam::channel::Sender <bool>, crossbeam::channel::Receiver <bool>)>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // does nothing, lets the os clean up the threads so nothing is blocked
        // unfortunately threads can't be manually killed so no manual clean up
        // isn't possible
        // the os cleans up the threads, right? I think it does
    }
}

impl Runtime {
    /// does a soft shutdown. This will conclude once all future tasks return as pending or get timed out. A
    /// blocking operation like std::time::sleep(...), if longer than the timeout duration, will be left
    /// dangling for the os to clean up (and as such .drop would never be called, so be careful). A
    /// soft blocking operation like .await will be properly shutdown assuming it yields to the executor at
    /// least once during the specified period.
    pub fn SoftShutdown (&mut self) {
        for _ in 0..self.threadPool.len() {
            let (thread, sender, receiver) = self.threadPool.remove(0);
            let _ = sender.send(true);  // not sure what could cause this to throw an error....

            // checking if the thread received the change (and there isn't being blocked)
            for _ in 0..12 {
                std::thread::sleep(std::time::Duration::from_millis(5));  // about 60ms total (is that enough? too much?)
                if receiver.try_recv().is_err() { continue; }  // the thread was unable to exit in time
                let _ = thread.join();
                break;
            }
        }
    }

    /// Does a non-blocking polling over every task in the runtime. In other words, Runtime::Poll prunes
    /// completed tasks from the thread-pool. Returns true if anything was pruned.
    pub fn Poll (&mut self) -> bool {
        for i in 0..self.threadPool.len() {
            let (_thread, _sender, receiver) = &self.threadPool[i];
            if let Ok(_completionCode) = receiver.try_recv() {  // doesn't matter if the code is true or false
                let completed = self.threadPool.remove(i);
                let _ = completed.0.join();
                return true;
            }
        } false
    }

    /// checks and returns if or if not the thread pool is empty.
    pub fn is_empty (&self) -> bool {
        self.threadPool.is_empty()
    }

    /// Spawns a new thread which handles all the polling. **Warning** This will spawn a new thread so
    /// don't call this in a loop (instead do something like placing the whole loop in as a single future).
    pub fn AddTask (&mut self, task: Pin <Box<dyn Future <Output=()> + Send>>) {
        let (sender, exitReceiver) = crossbeam::channel::bounded(1);
        let (completionSender, receiver) = crossbeam::channel::bounded(1);
        let thread = self.GetManagerThread(exitReceiver, completionSender, task);
        self.threadPool.push((thread, sender, receiver));
    }

    fn GetManagerThread(&self,
                        exitReceiver: crossbeam::channel::Receiver <bool>,
                        completionSender: crossbeam::channel::Sender <bool>,
                        mut task: Pin <Box<dyn Future <Output=()> + Send>>,
    ) -> std::thread::JoinHandle <()> {
        std::thread::spawn(move || {
            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);

            loop {
                if let Ok(_exitCode) = exitReceiver.try_recv() {
                    let _ = completionSender.send(true);
                    break;  // doesn't matter if the exit code is true or false
                }

                match task.as_mut().poll(&mut cx) {
                    Poll::Ready(_) => break completionSender.send(true).unwrap(),
                    Poll::Pending => {},
                }
            }
        })
    }
}

