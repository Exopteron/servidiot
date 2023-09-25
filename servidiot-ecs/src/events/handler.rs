use std::{
    any::TypeId,
    num::NonZeroUsize,
    sync::{atomic::AtomicU32, Arc, Exclusive},
    thread::{self, available_parallelism, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::bail;
use flume::{Receiver, Sender};
use fnv::{FnvHashMap, FnvHashSet};
use parking_lot::RwLock;

use super::{EventCollection, EventWrapper};

type CallbackFn<T, S> = dyn Fn(&S, &T) -> anyhow::Result<()> + Send + Sync;
pub struct EventManager<T: EventCollection, S> {
    callbacks: FnvHashMap<TypeId, Vec<Box<CallbackFn<T, S>>>>,
    event_channel: (Sender<T>, Receiver<T>),
}

impl<T: EventCollection, S: 'static> Default for EventManager<T, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: EventCollection, S: 'static> EventManager<T, S> {
    pub fn new() -> Self {
        Self {
            callbacks: FnvHashMap::default(),
            event_channel: flume::unbounded(),
        }
    }

    pub fn recv_channel(&self) -> &Receiver<T> {
        &self.event_channel.1
    }

    pub fn add_callback<E: EventWrapper<T> + 'static>(
        &mut self,
        callback: impl Fn(&S, &E) -> anyhow::Result<()> + 'static + Send + Sync,
    ) {
        self.callbacks
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(move |state, event| {
                callback(state, E::unwrap(event).expect("Should be the right type"))
            }));
    }

    pub fn submit_event<E: EventWrapper<T> + 'static>(&self, event: E) {
        self.event_channel.0.send(event.wrap()).unwrap();
    }

    pub fn handle_event(&self, state: &S, event: T) -> anyhow::Result<()> {
        if let Some(cbs) = self.callbacks.get(&event.inner_id()) {
            for cb in cbs {
                (cb)(state, &event)?;
            }
        } else {
            bail!("No event callback")
        }
        Ok(())
    }

    pub fn poll_one_event(&self, state: &S) -> anyhow::Result<()> {
        let v = self.event_channel.1.recv()?;
        self.handle_event(state, v)
    }

    pub fn poll_one_event_timeout(
        &self,
        state: &S,
        timeout: Duration,
    ) -> Result<anyhow::Result<()>, flume::RecvTimeoutError> {
        let v = self.event_channel.1.recv_timeout(timeout)?;
        Ok(self.handle_event(state, v))
    }
}

enum ThreadCommand<T: EventCollection, S> {
    Shutdown,
    AddTicker(Ticker<T, S>),
    RemoveTicker(TickerID, oneshot::Sender<Ticker<T, S>>),
}

struct ThreadHandle<T: EventCollection, S> {
    handle: JoinHandle<()>,
    communicator: Sender<ThreadCommand<T, S>>,
    tickers: RwLock<FnvHashSet<TickerID>>,
}

impl<T: EventCollection + 'static, S: 'static> ThreadHandle<T, S> {
    pub fn new(cb: impl FnOnce(Receiver<ThreadCommand<T, S>>) + Send + Sync + 'static) -> Self {
        let (communicator, recv) = flume::unbounded();

        let handle = thread::spawn(move || {
            cb(recv);
        });

        Self {
            handle,
            communicator,
            tickers: Default::default(),
        }
    }

    fn send_command(&self, c: ThreadCommand<T, S>) {
        self.communicator.send(c).expect("Must have died");
    }

    pub fn add_ticker(&self, t: Ticker<T, S>) {
        self.tickers.write().insert(t.id);
        self.send_command(ThreadCommand::AddTicker(t));
    }

    pub fn shutdown(self) {
        self.send_command(ThreadCommand::Shutdown);
        self.handle.join().expect("Panicked");
    }
}

pub type TickerID = u32;
pub type TickerCallback<T, S> = dyn FnMut(&EventManager<T, S>, &S) + Send + Sync;
struct Ticker<T: EventCollection, S> {
    id: TickerID,
    interval: Duration,
    last_tick: Instant,
    callback: Exclusive<Box<TickerCallback<T, S>>>,
}

impl<T: EventCollection, S> Ticker<T, S> {
    pub fn tick_if_needed(&mut self, mgr: &EventManager<T, S>, s: &S) {
        let now = Instant::now();

        if now >= (self.last_tick + self.interval) {
            (self.callback.get_mut())(mgr, s);
            self.last_tick = now;
        }
    }

    pub fn ticks_in(&mut self, mgr: &EventManager<T, S>, s: &S) -> Duration {
        self.tick_if_needed(mgr, s);
        (self.last_tick + self.interval) - Instant::now()
    }
}

pub struct ThreadedEventPool<T: EventCollection + 'static, S: Send + Sync + 'static> {
    event_handler: Arc<EventManager<T, S>>,
    threads: Vec<ThreadHandle<T, S>>,
    last_ticker_id: AtomicU32,
}

impl<T: EventCollection + 'static, S: Send + Sync + 'static> ThreadedEventPool<T, S> {
    pub fn event_handler(&self) -> Arc<EventManager<T, S>> {
        Arc::clone(&self.event_handler)
    }

    /// Tickers do not implement work stealing. Fire events off from here to do so.
    pub fn add_ticker(
        &self,
        interval: Duration,
        ticker: impl FnMut(&EventManager<T, S>, &S) + Send + Sync + 'static,
    ) -> TickerID {
        let ticker = Ticker {
            id: self
                .last_ticker_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            interval,
            last_tick: Instant::now(),
            callback: Exclusive::new(Box::new(ticker)),
        };
        let id = ticker.id;

        // find the thread with the lowest num. of tickers
        let mut lowest: Option<(usize, usize)> = None;
        for (index, thread) in self.threads.iter().enumerate() {
            match &mut lowest {
                Some((read_index, num_tickers)) => {
                    let thread_tickers = thread.tickers.read().len();
                    if thread_tickers < *num_tickers {
                        *read_index = index;
                        *num_tickers = thread_tickers;
                    }
                }
                None => lowest = Some((index, thread.tickers.read().len())),
            }
        }

        let (index, _) = lowest.expect("at least one thread should exist");

        self.threads[index].add_ticker(ticker);

        id
    }

    
    pub fn new(
        num_threads: Option<NonZeroUsize>,
        event_handler: EventManager<T, S>,
        state: Arc<S>,
    ) -> Self {
        let event_handler = Arc::new(event_handler);

        let mut threads = vec![];
        for _ in 0..(num_threads
            .unwrap_or(available_parallelism().expect("Couldn't get available parallelism"))
            .get())
        {
            let state = state.clone();
            let event_handler = event_handler.clone();
            threads.push(ThreadHandle::new(move |recv| {
                let mut tickers: FnvHashMap<TickerID, Ticker<T, S>> = Default::default();

                let mut should_break = false;
                loop {
                    if should_break {
                        break;
                    }
                    let has_ticker = !tickers.is_empty();
                    let mut next_ticker = Duration::MAX;
                    for ticker in tickers.values_mut() {
                        let ticks_in = ticker.ticks_in(&event_handler, &state);
                        if ticks_in < next_ticker {
                            next_ticker = ticks_in;
                        }
                    }

                    let selector = flume::Selector::new()
                        .recv(&recv, |thread_command| match thread_command.unwrap() {
                            ThreadCommand::Shutdown => should_break = true,
                            ThreadCommand::AddTicker(ticker) => {
                                tickers.insert(ticker.id, ticker);
                            }
                            ThreadCommand::RemoveTicker(id, shot) => {
                                shot.send(tickers.remove(&id).expect("invalid ticker command"))
                                    .expect("fail send");
                            }
                        })
                        .recv(event_handler.recv_channel(), |event| {
                            if let Err(e) = event_handler.handle_event(&state, event.unwrap()) {
                                panic!("handle later: {:?}", e);
                            }
                        });

                    if !has_ticker {
                        selector.wait();
                    } else {
                        let _ = selector.wait_deadline(Instant::now() + next_ticker);
                    }
                }
            }));
        }

        Self {
            event_handler,
            threads,
            last_ticker_id: AtomicU32::new(0),
        }
    }
}

impl<T: EventCollection, S: Send + Sync> Drop for ThreadedEventPool<T, S> {
    fn drop(&mut self) {
        for v in std::mem::take(&mut self.threads) {
            v.shutdown();
        }
    }
}
