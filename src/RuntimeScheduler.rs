use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::pin::Pin;

fn wake(_data: *const ()) {}
fn noop(_data: *const ()) {}

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(|data| RawWaker::new(data, &VTABLE), wake, wake, noop);

#[derive(Debug, Default)]
pub struct Runtime {
    // sender is for indicating a required exit; receiver is for checking if completed
    threadPool: Vec <(std::thread::JoinHandle <()>, crossbeam::channel::Sender <bool>, crossbeam::channel::Receiver <bool>)>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // does nothing, lets the os clean up the threads so nothing is blocked
    }
}

impl Runtime {
    pub fn Shutdown (&mut self) {
        for _ in 0..self.threadPool.len() {
            let (thread, sender, _receiver) = self.threadPool.remove(0);
            sender.send(true).unwrap();  // not sure what could cause this to throw an error....
            let _ = thread.join();
        }
    }

    /// Does a non-blocking polling over every task in the runtime. In other words, Runtime::Poll prunes
    /// completed tasks from the thread-pool. Returns true if anything was pruned.
    pub fn Poll (&mut self) -> bool {
        let mut removed = 0;
        for i in 0..self.threadPool.len() {
            let (_thread, _sender, receiver) = &self.threadPool[i - removed];
            if let Ok(_completionCode) = receiver.try_recv() {  // doesn't matter if the code is true or false
                let completed = self.threadPool.remove(i - removed);
                let _ = completed.0.join();
                removed += 1;
            }
        } removed > 0
    }

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
            let waker = RawWaker::new(std::ptr::null(), &VTABLE);
            let waker = unsafe { Waker::from_raw(waker) };
            let mut cx = Context::from_waker(&waker);

            loop {
                if let Ok(_exitCode) = exitReceiver.try_recv() {
                    break;  // does matter if the exit code is true or false
                }

                match task.as_mut().poll(&mut cx) {
                    Poll::Ready(_) => break completionSender.send(true).unwrap(),
                    Poll::Pending => {},
                }
            }
        })
    }
}
