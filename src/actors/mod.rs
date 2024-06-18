mod actor;
mod heartbeat;
mod broker;
mod message;
mod actor2;

#[macro_export] macro_rules! boxed_async {
    ($body:expr) => {
        Box::pin(async move { $body })
    };
}