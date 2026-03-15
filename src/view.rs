use iced::{
    Alignment, Element, Font, Length, never, padding,
    widget::{
        self, Container, button, column, container, rich_text, row, rule, span, stack,
        text::{self, Wrapping}
    }
};
use jiff::SignedDuration;
use lucide_icons::Icon;

use crate::{Annoyodoro, AppState, Message, circular::Circular, work_timer::WorkTimer};

pub const SPACING: f32 = 5.0;
pub const TIMER_TEXT_SIZE: f32 = 110.0;
pub const BIG_TEXT: f32 = 18.0;

impl Annoyodoro {
    pub fn view(&self) -> Element<'_, Message> {
        match self.state {
            AppState::Running { work_timer, .. } if work_timer.duration_remaning().is_zero() => {
                "If you're seeing this, the break timer didn't spawn and it's a bug".into()
            }
            AppState::Running {
                long_break_in,
                work_timer,
                last_work_session,
                ..
            } => self.main_view(long_break_in, work_timer, last_work_session),
            AppState::InitialWorkGoalPrompt { ref goal } => Self::initial_work_goal_prompt(goal)
        }
    }

    fn main_view(
        &self,
        long_break_in: u16,
        work_timer: WorkTimer,
        last_work_session: bool
    ) -> Element<'_, Message> {
        let palette = self.config.theme().palette();
        let time_left = work_timer
            .duration_remaning()
            .try_into()
            .unwrap_or(SignedDuration::MAX);
        let time_left = rich_text![
            span(time_left.as_mins().to_string()).color(palette.primary),
            span(":").color(
                self.config
                    .theme()
                    .extended_palette()
                    .background
                    .strongest
                    .color
            ),
            span(format!("{:02}", time_left.as_secs().abs() % 60)).color(palette.primary)
        ]
        .on_link_click(never)
        .size(TIMER_TEXT_SIZE);
        let toggle_pause_button_icon = if work_timer.is_paused() {
            Icon::Play
        } else {
            Icon::Pause
        };
        let toggle_pause_button = button(
            widget::text(toggle_pause_button_icon.unicode())
                .font(Font::with_name("lucide"))
                .size(BIG_TEXT)
        )
        .on_press(Message::TogglePause);

        let timer = stack![
            Circular {
                percentage: 1.0
                    - work_timer.duration_remaning().as_millis() as f32
                        / self.config.pomodoro.work_duration.as_millis() as f32,
                color: palette.primary,
                theme: self.config.theme()
            },
            container(time_left).center(Length::Fill),
            container(toggle_pause_button)
                .align_right(Length::Fill)
                .center_y(Length::Fill)
                .padding(padding::right(SPACING * 5.0)),
        ];
        let column = column![
            widget::checkbox(last_work_session)
                .on_toggle(|_| Message::ToggleLastWorkSession)
                .label("Last work session")
                .text_size(BIG_TEXT)
                .size(BIG_TEXT),
            rule::horizontal(2.0),
            row![
                container("Next long break in").width(Length::Fill),
                container(widget::text(format!("{long_break_in} pomodori")).color(palette.primary))
                    .align_right(Length::Fill)
            ],
            row![
                container("Pomodori today").width(Length::Fill),
                container(
                    widget::text(self.stats.pomodori_daily().to_string()).color(palette.primary)
                )
                .align_right(Length::Fill)
            ],
            row![
                "Current work goal",
                container(
                    widget::text(&self.stats.work_goals().last().unwrap().1)
                        .wrapping(Wrapping::WordOrGlyph)
                )
                .align_right(Length::Fill)
            ],
            self.error
                .as_ref()
                .map(|e| widget::text(e).style(text::danger))
        ]
        .spacing(SPACING)
        .width(TIMER_TEXT_SIZE * 4.0);
        #[cfg(debug_assertions)]
        let column = column.push(
            button("Early break (enabled only in debug mode)").on_press(Message::DebugEarlyBreak)
        );

        let content = row![timer, column]
            .spacing(SPACING * 2.0)
            .align_y(Alignment::Center);
        Container::new(content).center(Length::Fill).into()
    }

    fn initial_work_goal_prompt<'a>(work_goal: &str) -> Element<'a, Message> {
        let text_input = sweeten::text_input("Work goal", work_goal)
            .id("work-goal")
            .on_blur(Message::FocusTextInput)
            .on_input(Message::InitialWorkGoalChange)
            .on_submit(Message::InitialWorkGoalSubmit);
        let column = column!["Enter the goal of your fist work session", text_input]
            .align_x(Alignment::Center)
            .max_width(TIMER_TEXT_SIZE * 3.0);
        Container::new(column)
            .padding(SPACING)
            .center(Length::Fill)
            .into()
    }
}
