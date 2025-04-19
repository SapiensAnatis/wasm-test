use crate::log::*;
use futures::Future;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::convert::TryInto;
use std::fmt;
use std::num::Saturating;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Waker;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

pub struct Connection {
    url: String,
}

impl Connection {
    pub fn new(url: &str) -> Self {
        Self {
            url: String::from(url),
        }
    }

    pub fn connect(self: &mut Self) -> Result<HandshakeFuture, &'static str> {
        let ws = match WebSocket::new(self.url.as_str()) {
            Ok(ws) => ws,
            Err(_) => {
                return Err("Failed to create websocket");
            }
        };

        let fut = HandshakeFuture::new(ws);

        return Ok(fut);
    }
}

#[derive(Copy, Clone)]
enum HandshakeStatus {
    InProgress,
    Complete,
    Error,
}

impl fmt::Display for HandshakeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandshakeStatus::InProgress => write!(f, "IN_PROGRESS"),
            HandshakeStatus::Complete => write!(f, "COMPLETE"),
            HandshakeStatus::Error => write!(f, "ERROR"),
        }
    }
}

pub struct HandshakeFuture {
    websocket: WebSocket,
    on_message: Closure<dyn FnMut(MessageEvent) -> ()>,
    status: Rc<RefCell<HandshakeStatus>>,
    waker: Rc<RefCell<Option<Waker>>>,
}

impl HandshakeFuture {
    pub fn new(websocket: WebSocket) -> Self {
        let status_rc = Rc::new(RefCell::new(HandshakeStatus::InProgress));
        let status_rc_clone = status_rc.clone();

        let waker_rc = Rc::new(RefCell::new(None));
        let waker_rc_clone = waker_rc.clone();

        let on_message: Closure<dyn FnMut(_)> = Closure::new(move |e: MessageEvent| {
            Self::on_message(e, status_rc_clone.borrow_mut(), waker_rc_clone.borrow_mut())
        });

        websocket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        Self {
            websocket,
            status: status_rc,
            on_message,
            waker: waker_rc,
        }
    }

    fn on_message(
        e: MessageEvent,
        mut status: RefMut<HandshakeStatus>,
        mut waker_opt: RefMut<Option<Waker>>,
    ) {
        if let Ok(_abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            unimplemented!("Array buffers are not implemented");
        } else if let Ok(_blob) = e.data().dyn_into::<web_sys::Blob>() {
            unimplemented!("Blobs are not implemented");
        } else if let Ok(string) = e.data().dyn_into::<js_sys::JsString>() {
            console_log!("connected: {}", string);
        } else {
            unimplemented!("What the hell");
        }

        *status = HandshakeStatus::Complete;

        let waker = waker_opt.deref_mut();

        if let Some(waker2) = waker {
            console_log!("Joe Biden wake up");
            waker2.wake_by_ref();
        } else {
            console_log!("I sleep");
        }
    }
}

impl Future for HandshakeFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let status = *self.status.borrow();

        console_log!("Polled status: {}", status);

        match status {
            HandshakeStatus::InProgress => {
                self.waker.replace(Some(cx.waker().clone()));
                Poll::Pending
            }
            HandshakeStatus::Complete => Poll::Ready(Ok(())),
            HandshakeStatus::Error => Poll::Ready(Err(String::from("Handshake failed"))),
        }
    }
}
