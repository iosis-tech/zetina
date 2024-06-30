use futures::{Future, FutureExt};
use std::pin::Pin;
use tokio::sync::mpsc;

pub struct Process<'future, PR> {
    future: Pin<Box<dyn Future<Output = PR> + Send + 'future>>,
    abort: mpsc::Sender<()>,
}

impl<'future, PR> Process<'future, PR> {
    pub fn new(
        future: Pin<Box<dyn Future<Output = PR> + Send + 'future>>,
        abort: mpsc::Sender<()>,
    ) -> Self {
        Self { future, abort }
    }

    pub async fn abort(&self) -> Result<(), mpsc::error::SendError<()>> {
        self.abort.send(()).await
    }

    pub fn into_parts(self) -> (Pin<Box<dyn Future<Output = PR> + 'future>>, mpsc::Sender<()>) {
        (self.future, self.abort)
    }
}

impl<'future, PR> Future for Process<'future, PR> {
    type Output = PR;
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.future.poll_unpin(cx)
    }
}
