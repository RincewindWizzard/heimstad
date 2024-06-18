mod actor;
mod heartbeat;
mod broker;
mod message;

#[macro_export] macro_rules! boxed_async {
    ($body:expr) => {
        Box::pin(async move { $body })
    };
}