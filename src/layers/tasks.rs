use crate::scope::use_layer;
use futures::Future;
use std::{any::Any, pin::Pin};
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        watch,
    },
    task::JoinHandle,
};

pub type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type Message = Box<dyn Any + Send + 'static>;

pub struct AsyncTasks {
    pub task: Option<JoinHandle<()>>,
    pub layer: TaskLayer,
    pub message_rx: Option<UnboundedReceiver<Message>>,
    pub shutdown_sender: Option<watch::Sender<()>>,
}

impl AsyncTasks {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel::<Message>();
        let (task_tx, mut task_rx) = mpsc::unbounded_channel::<Task>();
        let (shutdown_tx, mut shutdown_rx) = watch::channel::<()>(());

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    },
                    Some(job) = task_rx.recv() => {
                        tokio::select! {
                            _ = job => {},
                            _ = shutdown_rx.changed() => {
                                break;
                            },
                        }
                    }
                }
            }
        });

        Self {
            layer: TaskLayer {
                dispatcher: Dispatcher(message_tx),
                task: task_tx,
            },
            task: Some(handle),
            message_rx: Some(message_rx),
            shutdown_sender: Some(shutdown_tx),
        }
    }

    pub async fn reciever(&mut self) -> UnboundedReceiver<Message> {
        let reciever = self.message_rx.take().expect("Already taken reciever");
        reciever
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }
        self.task.take().expect("Missing task handle").await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct TaskLayer {
    dispatcher: Dispatcher,
    task: UnboundedSender<Task>,
}

impl TaskLayer {
    pub fn send_with_dispatch<Fu>(&self, fut: impl Fn(Dispatcher) -> Fu + 'static)
    where
        Fu: Future<Output = ()> + Send + 'static,
    {
        let fut = fut(self.dispatcher.clone());
        let _ = self.task.send(Box::pin(fut));
    }
}

pub fn async_with_dispatch<Fu>(fut: impl Fn(Dispatcher) -> Fu + 'static)
where
    Fu: Future<Output = ()> + Send + 'static,
{
    let sync = use_layer::<TaskLayer>();
    sync.send_with_dispatch(fut);
}

#[derive(Clone)]
pub struct Dispatcher(mpsc::UnboundedSender<Message>);

impl Dispatcher {
    pub fn dispatch<T: Send + Any + 'static>(&self, x: T) {
        let _ = self.0.send(Box::new(x));
    }
}
