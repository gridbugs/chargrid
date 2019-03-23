extern crate prototty;
use prototty::*;

pub enum ControlFlow {
    Exit,
}

pub struct AppState {
    scroll_state: VerticalScrollState,
    text: String,
}

impl AppState {
    pub fn new(text: String) -> Self {
        Self {
            scroll_state: VerticalScrollState::new(),
            text,
        }
    }
    pub fn tick<I>(&mut self, inputs: I, view: &AppView) -> Option<ControlFlow>
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        self.scroll_state = view.scroll().state();
        for input in inputs {
            match input {
                prototty_inputs::ETX
                | prototty_inputs::ESCAPE
                | ProtottyInput::Char('q') => {
                    return Some(ControlFlow::Exit);
                }
                ProtottyInput::MouseScroll { direction, .. } => match direction {
                    ScrollDirection::Up => self.scroll_state.scroll_up_line(),
                    ScrollDirection::Down => self.scroll_state.scroll_down_line(),
                    _ => (),
                },
                ProtottyInput::Up => self.scroll_state.scroll_up_line(),
                ProtottyInput::Down => self.scroll_state.scroll_down_line(),
                ProtottyInput::PageUp => self.scroll_state.scroll_up_page(),
                ProtottyInput::PageDown => self.scroll_state.scroll_down_page(),
                ProtottyInput::Home | ProtottyInput::Char('g') => {
                    self.scroll_state.scroll_to_top()
                }
                ProtottyInput::End | ProtottyInput::Char('G') => {
                    self.scroll_state.scroll_to_bottom()
                }
                _ => (),
            }
        }
        None
    }
}

type Scroll = VerticalScrolled<Identity<RichTextView<wrap::Word>>>;

pub struct AppView {
    view: Aligned<Bordered<Bounded<Scroll>>>,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            view: decorate(RichTextView::new(wrap::Word::new()))
                .vertical_scroll(VerticalScrollbar::default())
                .bound(Size::new(40, 40))
                .border(Border {
                    title_style: Style {
                        bold: Some(true),
                        foreground: Some(rgb24(0, 255, 0)),
                        background: Some(rgb24(0, 64, 0)),
                        ..Default::default()
                    },
                    padding: BorderPadding {
                        right: 0,
                        left: 2,
                        top: 1,
                        bottom: 1,
                    },
                    ..Border::default_with_title("Pager")
                })
                .centre(),
        }
    }
    fn scroll_mut(&mut self) -> &mut Scroll {
        &mut self.view.view.view.view
    }
    fn scroll(&self) -> &Scroll {
        &self.view.view.view.view
    }
}

impl<'a> View<&'a AppState> for AppView {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        app_state: &'a AppState,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let rich_text = &[
            ("Hello, World!\nblah\nblah blah ", Style::default()),
            (
                "blue\n",
                Style {
                    foreground: Some(colours::BRIGHT_BLUE),
                    bold: Some(true),
                    ..Default::default()
                },
            ),
            ("User string:\n", Default::default()),
            (
                app_state.text.as_ref(),
                Style {
                    background: Some(colours::RED),
                    underline: Some(true),
                    ..Default::default()
                },
            ),
        ];
        self.scroll_mut()
            .sync_scroll_position(&app_state.scroll_state);
        self.view.view(rich_text, context, grid);
    }
}
