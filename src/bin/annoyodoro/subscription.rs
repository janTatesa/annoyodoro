use std::{iter, time::Duration};

use iced::{
    Event, Subscription,
    event::{self, Status},
    keyboard::{self, Key},
    window::Id
};
use itertools::chain;

use crate::{Annoyodoro, Message};

impl Annoyodoro {
    fn toggle_pause_subscription(event: Event, _: Status, _: Id) -> Option<Message> {
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Character(key),
            modifiers,
            ..
        }) = event
            && key == "p"
            && modifiers.is_empty()
        {
            return Some(Message::TogglePause)
        }

        None
    }

    fn retry_subscription(event: Event, _: Status, _: Id) -> Option<Message> {
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Character(key),
            modifiers,
            ..
        }) = event
            && key == "r"
            && modifiers.is_empty()
        {
            return Some(Message::Retry)
        }

        None
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let iter = chain![
            self.error
                .is_some()
                .then(|| event::listen_with(Self::retry_subscription)),
            (!self.work_timer.is_paused() && !self.break_time)
                .then(|| iced::time::every(Duration::from_secs(1)).map(Message::Tick)),
            iter::once(event::listen_with(Self::toggle_pause_subscription))
        ];

        Subscription::batch(iter)
    }
}
