use std::io;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};

use crate::app;

pub struct UiState {
    lists: [ListState; 2],
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            lists: [ListState::default(), ListState::default()],
        }
    }
}

pub struct Ui<B: Backend> {
    terminal: Terminal<B>,

    frame_layout: Layout,
    list_layout: Layout,
}

impl<B: Backend> Ui<B> {
    pub fn new(terminal: Terminal<B>) -> Ui<B> {
        Ui {
            terminal,

            frame_layout: Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Min(5),
                        // Should be 1, but: https://github.com/fdehau/tui-rs/issues/366
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                ),

            list_layout: Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints(
                    [Constraint::Percentage(50), Constraint::Percentage(50)]
                        .as_ref(),
                ),
        }
    }

    // TODO: Is there a better way of doing this? It's a bit horrid.
    // https://github.com/fdehau/tui-rs/issues/412
    pub fn list_height(&mut self) -> u16 {
        let frame = self.terminal.get_frame();
        let frame_chunks = self.frame_layout.split(frame.size());
        let files_block = files_block();
        let files_block_inner_area = files_block.inner(frame_chunks[0]);
        let list_chunks = self.list_layout.split(files_block_inner_area);
        list_chunks[0].height
    }

    pub fn draw(
        &mut self,
        state: &app::State,
        ui_state: &mut UiState,
    ) -> io::Result<()> {
        let frame_layout = self.frame_layout.clone();
        let list_layout = self.list_layout.clone();

        self.terminal.draw(|f| {
            let frame_chunks = frame_layout.split(f.size());

            let files_block = files_block();
            let files_block_inner_area = files_block.inner(frame_chunks[0]);
            f.render_widget(files_block, frame_chunks[0]);

            {
                let list_chunks = list_layout.split(files_block_inner_area);

                Ui::draw_file_list(
                    f,
                    list_chunks[0],
                    &state,
                    &mut ui_state.lists[0],
                    0,
                );
                Ui::draw_file_list(
                    f,
                    list_chunks[1],
                    &state,
                    &mut ui_state.lists[1],
                    1,
                );
            }

            Ui::draw_status(f, frame_chunks[1], &state);
        })
    }

    fn draw_file_list(
        f: &mut Frame<B>,
        area: Rect,
        state: &app::State,
        mut list_state: &mut ListState,
        thing: usize,
    ) {
        let list_block = Block::default().borders(Borders::NONE);

        list_state.select(state.selected_file_in_list(thing));
        let files = state.files_in_list(thing);
        let list = (match files {
            None => List::new(vec![]),
            Some(files) => List::new(
                files
                    .iter()
                    .map(|f| {
                        ListItem::new(vec![Spans::from(vec![if f.is_dir() {
                            Span::styled(
                                format!("{:<9}", &f.name()),
                                Style::default()
                                    .fg(Color::LightBlue)
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::raw(f.name())
                        }])])
                    })
                    .collect::<Vec<ListItem>>(),
            ),
        })
        .block(list_block)
        .highlight_style(Style::default().bg(Color::Rgb(32, 32, 32)));

        f.render_stateful_widget(list, area, &mut list_state);
    }

    fn draw_status(f: &mut Frame<B>, area: Rect, state: &app::State) {
        let status_block = Block::default().borders(Borders::NONE);
        let para = Paragraph::new(state.base_path())
            .block(status_block)
            .alignment(Alignment::Left);
        f.render_widget(para, area);
    }
}

fn files_block() -> Block<'static> {
    Block::default().title("Files").borders(Borders::ALL)
}
