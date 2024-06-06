use futures::Future;
use std::{any::Any, pin::Pin, sync::Arc};
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    task::JoinHandle,
};

use crate::{
    environment::*,
    nodes::{IntoScope, ReactiveNode, Scope},
    signal::*,
};

pub type Message = (Scope, Box<dyn Any + Send + 'static>);
pub type Task = (Scope, Pin<Box<dyn Future<Output = ()> + Send>>);

#[derive(Debug, Clone)]
pub struct TaskRunner {
    message_rx: Arc<Mutex<mpsc::UnboundedReceiver<Message>>>,
    message_tx: mpsc::UnboundedSender<Message>,
    cancel_tx: broadcast::Sender<Scope>,
    shutdown_tx: broadcast::Sender<()>,
    task_tx: mpsc::UnboundedSender<Task>,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl TaskRunner {
    pub async fn new() -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel::<Message>();
        let (task_tx, mut task_rx) = mpsc::unbounded_channel::<Task>();
        let (cancel_tx, _) = broadcast::channel::<Scope>(100);
        let (shutdown_tx, _) = broadcast::channel::<()>(100);

        let shutdown_tx_c = shutdown_tx.clone();
        let task_handle = tokio::spawn(async move {
            loop {
                let shutdown_tx = shutdown_tx_c.clone();
                let mut shutdown_rx = shutdown_tx.subscribe();
                tokio::select! {
                    task = task_rx.recv() => {
                        if let Some(task) = task {
                            Self::spawn_task(task, shutdown_tx).await
                        }
                    },
                    _ = shutdown_rx.recv() => {
                        break
                    }
                }
            }
        });

        Self {
            message_rx: Arc::new(Mutex::new(message_rx)),
            message_tx,
            cancel_tx,
            shutdown_tx,
            task_tx,
            task_handle: Arc::new(Mutex::new(Some(task_handle))),
        }
    }

    async fn spawn_task(task: Task, shutdown: broadcast::Sender<()>) {
        tokio::spawn(async move {
            let mut shutdown_rx = shutdown.subscribe();
            tokio::select! {
                _ = task.1 => {},
                _ = shutdown_rx.recv() => {}
            }
        });
    }

    async fn await_cancel(cancel_tx: broadcast::Sender<Scope>, task_id: Scope) {
        let mut cancel = cancel_tx.subscribe();
        loop {
            if let Ok(id) = cancel.recv().await {
                if task_id == id {
                    break;
                }
            }
        }
    }

    pub async fn listen(&self) {
        if let Some((_, message)) = self.message_rx.lock().await.recv().await {
            let message = message as Box<dyn Any + 'static>;
            send_boxed(&message);
        }
    }

    pub async fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
        if let Some(handle) = self.task_handle.lock().await.take() {
            let _ = handle.await;
        };
    }
}

#[derive(Clone)]
pub struct TaskMessageTransmitter(Scope, mpsc::UnboundedSender<Message>);
impl TaskMessageTransmitter {
    pub fn send<T: Send + Any + 'static>(&self, x: T) {
        let _ = self.1.send((self.0, Box::new(x)));
    }
}

pub async fn create_async_scope<T, F>(f: impl FnOnce(TaskRunner) -> F)
where
    F: Future<Output = T>,
{
    let scope = with_runtime(|r| r.nodes.insert(ReactiveNode::default()));
    let previous = with_runtime(|r| r.tracker.replace(Some(scope)));

    let runtime = TaskRunner::new().await;
    provide_context(runtime.clone());

    f(runtime).await;

    with_runtime(|s| s.tracker.replace(previous));
    with_runtime(|r| {
        r.tracker.replace(previous);
        r.cleanup_child_scope(scope);
        r.nodes.dispose(scope);
    });
}

pub fn create_async_task<Value, Fu>(
    arg: impl SignalGet<Value> + 'static,
    fut: impl Fn(Value, TaskMessageTransmitter) -> Fu + 'static,
) -> TaskControl
where
    Value: Clone + 'static,
    Fu: Future<Output = ()> + Send + 'static,
{
    let status = create_signal(TaskState::Initial);
    let tasks = use_context::<TaskRunner>();

    let canceller = tasks.cancel_tx.clone();
    let message_tx = tasks.message_tx.clone();

    let id = create_memo(move || {
        let data = arg.get();

        let id = with_runtime(|s| s.tracker.get()).unwrap();

        let canceller_c = canceller.clone();
        let message_tx = message_tx.clone();
        let inner = fut(data, TaskMessageTransmitter(id, message_tx.clone()));
        let future = Box::pin(async move {
            let _ = message_tx.send((id, Box::new((id, TaskState::Pending))));
            tokio::select! {
                _ = TaskRunner::await_cancel(canceller_c, id) => {
                    let _ = message_tx.send((id, Box::new((id, TaskState::Cancelled))));
                },
                _ = inner => {
                    let _ = message_tx.send((id, Box::new((id, TaskState::Finnished))));
                },
            }
        });

        let _ = tasks.task_tx.send((id, future));
        let canceller = canceller.clone();
        on_cleanup(move || {
            let _ = canceller.send(id);
        })
    });

    let status_c = status.clone();
    on(move |(task_id, ev): &(Scope, TaskState)| {
        if *task_id != id.into_scope() {
            return;
        }
        status_c.set(*ev);
    });

    TaskControl {
        id: id.into_scope(),
        cancel_tx: tasks.cancel_tx.clone(),
        status,
    }
}

#[derive(Debug, Clone)]
pub struct TaskControl {
    id: Scope,
    cancel_tx: broadcast::Sender<Scope>,
    status: Signal<TaskState>,
}

impl TaskControl {
    pub fn get_state(&self) -> TaskState {
        self.status.get()
    }
    pub fn stop(&self) {
        let _ = self.cancel_tx.send(self.id);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TaskState {
    Initial,
    Pending,
    Cancelled,
    Finnished,
}
