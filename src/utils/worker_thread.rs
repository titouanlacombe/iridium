use std::{sync::mpsc, thread};

type Command<T> = Box<dyn FnOnce(&mut T, &mut bool) + Send>;

pub struct WorkerThread<ThreadData: Default + 'static> {
    sender: mpsc::Sender<Command<ThreadData>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl<ThreadData: Default + 'static> WorkerThread<ThreadData> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Command<ThreadData>>();
        Self {
            sender: tx,
            handle: Some(Self::spawn_thread(rx)),
        }
    }

    fn spawn_thread(rx: mpsc::Receiver<Command<ThreadData>>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut data = ThreadData::default();
            let mut stop = false;
            loop {
                let command = {
                    let _span = tracy_client::span!("Worker thread receive");
                    rx.recv()
                };

                {
                    let _span = tracy_client::span!("Worker thread execute");
                    command.unwrap()(&mut data, &mut stop);
                }

                if stop {
                    break;
                }
            }
        })
    }

    pub fn send(&self, command: Command<ThreadData>) {
        self.sender.send(command).unwrap();
    }
}

impl<ThreadData: Default + 'static> Drop for WorkerThread<ThreadData> {
    fn drop(&mut self) {
        self.send(Box::new(|_, stop| {
            *stop = true;
        }));

        self.handle.take().unwrap().join().unwrap();
    }
}
