mod actor;
mod heartbeat;

#[macro_export] macro_rules! boxed_async {
    ($body:expr) => {
        Box::pin(async move { $body })
    };
}