//! Show a circular progress indicator.
use std::f32::consts::PI;

use iced::{
    Color, Element, Event, Length, Radians, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        self, Clipboard, Layout, Shell, Widget, layout, renderer,
        widget::tree::{self, Tree}
    },
    mouse,
    widget::canvas::{self, LineCap},
    window
};

pub struct Circular {
    pub percentage: f32,
    pub color: Color,
    pub theme: Theme
}

#[derive(Default)]
struct State {
    cache: canvas::Cache
}

const SIZE: f32 = 400.0;
const WIDTH: f32 = 8.0;
impl<Message> Widget<Message, Theme, Renderer> for Circular
where
    Message: Clone
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(SIZE),
            height: Length::Fixed(SIZE)
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits
    ) -> layout::Node {
        layout::atomic(limits, SIZE, SIZE)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            state.cache.clear();
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle
    ) {
        use advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width() / 2.0 - WIDTH;
            let track_path = canvas::Path::circle(frame.center(), track_radius);
            frame.fill(
                &track_path,
                self.theme.extended_palette().background.weaker.color
            );

            frame.stroke(
                &track_path,
                canvas::Stroke::default()
                    .with_color(self.theme.extended_palette().background.weak.color)
                    .with_width(WIDTH)
            );

            let mut builder = canvas::path::Builder::new();

            builder.arc(canvas::path::Arc {
                center: frame.center(),
                radius: track_radius,
                start_angle: Radians(-PI * 0.5),
                end_angle: Radians(self.percentage * PI * 2.0 - PI * 0.5)
            });

            let bar_path = builder.build();

            frame.stroke(
                &bar_path,
                canvas::Stroke::default()
                    .with_color(self.color)
                    .with_width(WIDTH)
                    .with_line_cap(LineCap::Round)
            );
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            use iced::advanced::graphics::geometry::Renderer as _;

            renderer.draw_geometry(geometry);
        });
    }
}

impl<'a, Message, Theme> From<Circular> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Circular: iced::advanced::Widget<Message, Theme, iced::Renderer>
{
    fn from(circular: Circular) -> Self {
        Self::new(circular)
    }
}
