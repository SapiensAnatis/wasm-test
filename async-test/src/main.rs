use std::future::*;
use std::pin::Pin;
use std::task::*;

pub struct CounterFuture {
    counter: i32,
}

impl Future for CounterFuture {
    type Output = Result<(), &'static str>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.counter += 1;
        println!("Counter: {}", self.counter);

        match self.counter {
            ..5 => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            _ => Poll::Ready(Ok(())),
        }
    }
}

#[tokio::main]
async fn main() {
    let fut = CounterFuture { counter: 0 };
    match fut.await {
        Ok(()) => {
            println!("Future finished.");
        }
        Err(e) => {
            eprintln!("Future failed: {}", e);
        }
    };
}
