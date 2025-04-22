use crate::connection::{SignalRConnection, CHANNEL_BOUND_SIZE};
use crate::message::InvocationMessage;
use futures::channel::mpsc;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde_json::Value;
use wasm_bindgen_futures::spawn_local;

impl SignalRConnection {
    pub fn on<T1, T2>(&mut self, method_name: &str, handler: impl Fn(T1, T2) -> () + 'static)
    where
        T1: DeserializeOwned + 'static,
        T2: DeserializeOwned + 'static,
    {
        let (sender, mut receiver) = mpsc::channel::<InvocationMessage>(CHANNEL_BOUND_SIZE);

        {
            self.invocation_subscribers
                .borrow_mut()
                .insert(method_name.to_owned(), sender);
        }

        spawn_local(async move {
            while let Some(invocation) = receiver.next().await {
                if let Err(e) = Self::call_handler(invocation.arguments, &handler) {
                    console_error!("Failed to invoke handler: {}", e);
                }
            }
        })
    }

    fn call_handler<T1, T2>(
        mut args: Vec<Value>,
        handler: impl Fn(T1, T2) -> (),
    ) -> Result<(), String>
    where
        T1: DeserializeOwned,
        T2: DeserializeOwned,
    {
        let arg2: T2 = args
            .pop()
            .ok_or("Missing argument".to_owned())
            .and_then(|a| {
                serde_json::from_value(a)
                    .map_err(|e| format!("Failed to deserialize argument `{}`", e))
            })?;

        let arg1: T1 = args
            .pop()
            .ok_or("Missing argument".to_owned())
            .and_then(|a| {
                serde_json::from_value(a)
                    .map_err(|e| format!("Failed to deserialize argument `{}`", e))
            })?;

        handler(arg1, arg2);

        return Ok(());
    }
}
