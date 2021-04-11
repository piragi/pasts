#![forbid(unsafe_code)]

use async_std::task::sleep;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use pasts::Race;

/// Shared state between tasks on the thread.
struct State {
    counter: usize,
    one: Interval,
    two: Interval,
}

impl State {
    fn one(&mut self, _: ()) -> Poll<()> {
        println!("One {}", self.counter);
        self.counter += 1;
        if self.counter > 6 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn two(&mut self, _: ()) -> Poll<()> {
        println!("Two {}", self.counter);
        self.counter += 1;
        Poll::Pending
    }
}

struct Interval(Duration, Pin<Box<dyn Future<Output = ()>>>);

impl Interval {
    fn new(duration: Duration) -> Self {
        Interval(duration, Box::pin(sleep(duration)))
    }
}

impl Future for Interval {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match self.1.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => {
                self.1 = Box::pin(sleep(self.0));
                Poll::Ready(())
            }
        }
    }
}

async fn run() {
    let mut state = State {
        counter: 0,
        one: Interval::new(Duration::from_secs_f64(1.0)),
        two: Interval::new(Duration::from_secs_f64(2.0)),
    };

    loop {
        if let Poll::Ready(output) = Race::new(&mut state, |state, race| {
            race.when(&mut state.one, State::one)
                .when(&mut state.two, State::two)
        }).await {
            break output;
        }
    }
}

fn main() {
    pasts::block_on(run())
}
