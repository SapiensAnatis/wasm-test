use crate::log::*;
use futures::Future;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::num::Saturating;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
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

    pub fn connect(self: &mut Self) -> Result<HandshakeFuture, &'static str> {
        let ws = match WebSocket::new(self.url.as_str()) {
            Ok(ws) => ws,
            Err(_) => {
                return Err("Failed to create websocket");
            }
        };

        self.websocket = Some(ws.clone());

        let fut = HandshakeFuture::new(ws.clone());

        return Ok(fut);
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

#[derive(Copy, Clone)]
enum HandshakeStatus {
    InProgress,
    Complete,
    Error,
}

struct HandshakeFuture {
    websocket: WebSocket,
    on_message: Closure<dyn FnMut(MessageEvent) -> ()>,
    status: Rc<RefCell<HandshakeStatus>>,
}

impl HandshakeFuture {
    pub fn new(websocket: WebSocket) -> Self {
        let status_rc = Rc::new(RefCell::new(HandshakeStatus::InProgress));

        let mut result = Self {
            websocket,
            status: status_rc.clone(),
            on_message: Closure::new(move |e: MessageEvent| {
                Self::on_message(e, status_rc.borrow_mut())
            }),
        };

        result.init();

        result
    }

    fn init(self: &mut Self) {
        self.websocket
            .set_onmessage(Some(self.on_message.as_ref().unchecked_ref()));
    }

    fn on_message(e: MessageEvent, status: RefMut<HandshakeStatus>) {
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

impl Future for HandshakeFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let status = *self.status.borrow();

        match status {
            HandshakeStatus::InProgress => Poll::Pending,
            HandshakeStatus::Complete => Poll::Ready(Ok(())),
            HandshakeStatus::Error => Poll::Ready(Err(String::from("Handshake failed"))),
        }
    }
}
