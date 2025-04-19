use crate::log::*;
use futures::Future;
use std::num::Saturating;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

pub struct Connection {
    url: String,
    websocket: Option<WebSocket>,
}

impl Connection {
    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url),
            websocket: None,
        }
    }

    pub fn connect(self: &mut Self) -> Result<(), &'static str> {
        let ws = match WebSocket::new(self.url.as_str()) {
            Ok(ws) => ws,
            Err(_) => {
                return Err("Failed to create websocket");
            }
        };

        let on_message_closure = Closure::<dyn FnMut(_)>::new(Self::on_connect_response);

        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        ws.set_onmessage(Some(on_message_closure.as_ref().unchecked_ref()));

        // This is literally a memory leak, but otherwise JS attempts to call our function
        // after it has been dropped
        on_message_closure.forget();

        self.websocket = Some(ws);

        return Ok(());
    }

    fn on_connect_response(e: MessageEvent) {
        console_log!("{:?}", e);

        if let Ok(_abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            unimplemented!("Array buffers are not implemented");
        } else if let Ok(_blob) = e.data().dyn_into::<web_sys::Blob>() {
            unimplemented!("Blobs are not implemented");
        } else if let Ok(string) = e.data().dyn_into::<js_sys::JsString>() {
            console_log!("connected: {}", string);
        }
    }
}

enum HandshakeStatus {
    InProgress,
    Complete,
    Error(String),
}

struct HandshakeFuture<'a> {
    websocket: &'a WebSocket,
    on_message: Closure<dyn FnMut(MessageEvent) -> ()>,

    status: HandshakeStatus,
}

impl<'a> HandshakeFuture<'a> {
    fn new(websocket: &'a WebSocket) -> Self {
        Self {
            websocket: websocket,
            on_message: Closure::new(Self::on_message),
            status: HandshakeStatus::InProgress,
        }
    }

    fn on_message(e: MessageEvent) {
        console_log!("{:?}", e);

        if let Ok(_abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            unimplemented!("Array buffers are not implemented");
        } else if let Ok(_blob) = e.data().dyn_into::<web_sys::Blob>() {
            unimplemented!("Blobs are not implemented");
        } else if let Ok(string) = e.data().dyn_into::<js_sys::JsString>() {
            console_log!("connected: {}", string);
        }
    }
}

impl Future for HandshakeFuture<'_> {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &self.status {
            HandshakeStatus::InProgress => Poll::Pending,
            HandshakeStatus::Complete => Poll::Ready(Ok(())),
            HandshakeStatus::Error(e) => Poll::Ready(Err(e.clone())),
        }
    }
}
