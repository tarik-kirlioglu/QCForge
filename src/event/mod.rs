pub mod key;

use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::actions::Action;
use key::map_key_event;

pub struct EventHandler {
    _task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tx: UnboundedSender<Action>) -> Self {
        let task = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(Duration::from_millis(250));

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        if tx.send(Action::Render).is_err() {
                            break;
                        }
                    }
                    event = reader.next() => {
                        match event {
                            Some(Ok(CrosstermEvent::Key(key))) => {
                                if let Some(action) = map_key_event(key) {
                                    if tx.send(action).is_err() {
                                        break;
                                    }
                                }
                            }
                            Some(Ok(CrosstermEvent::Resize(w, h))) => {
                                if tx.send(Action::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            Some(Err(_)) | None => break,
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { _task: task }
    }
}
