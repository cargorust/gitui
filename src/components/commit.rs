use super::{
    visibility_blocking, CommandBlocking, CommandInfo, Component,
    DrawableComponent,
};
use crate::{
    keys,
    queue::{InternalEvent, NeedsUpdate, Queue},
    strings, ui,
};
use asyncgit::{sync, CWD};
use crossterm::event::{Event, KeyCode};
use log::error;
use std::borrow::Cow;
use strings::commands;
use sync::HookResult;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Text, Widget},
    Frame,
};

pub struct CommitComponent {
    msg: String,
    visible: bool,
    stage_empty: bool,
    queue: Queue,
}

impl DrawableComponent for CommitComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, _rect: Rect) {
        if self.visible {
            let txt = if self.msg.is_empty() {
                [Text::Styled(
                    Cow::from(strings::COMMIT_MSG),
                    Style::default().fg(Color::DarkGray),
                )]
            } else {
                [Text::Raw(Cow::from(self.msg.clone()))]
            };

            ui::Clear::new(
                Paragraph::new(txt.iter())
                    .block(
                        Block::default()
                            .title(strings::COMMIT_TITLE)
                            .borders(Borders::ALL),
                    )
                    .alignment(Alignment::Left),
            )
            .render(f, ui::centered_rect(60, 20, f.size()));
        }
    }
}

impl Component for CommitComponent {
    fn commands(
        &self,
        out: &mut Vec<CommandInfo>,
        _force_all: bool,
    ) -> CommandBlocking {
        out.push(CommandInfo::new(
            commands::COMMIT_OPEN,
            !self.stage_empty,
            !self.visible,
        ));
        out.push(CommandInfo::new(
            commands::COMMIT_ENTER,
            self.can_commit(),
            self.visible,
        ));
        out.push(CommandInfo::new(
            commands::CLOSE_POPUP,
            true,
            self.visible,
        ));
        visibility_blocking(self)
    }

    fn event(&mut self, ev: Event) -> bool {
        if self.visible {
            if let Event::Key(e) = ev {
                match e.code {
                    KeyCode::Esc => {
                        self.hide();
                    }
                    KeyCode::Char(c) => {
                        self.msg.push(c);
                    }
                    KeyCode::Enter if self.can_commit() => {
                        self.commit();
                    }
                    KeyCode::Backspace if !self.msg.is_empty() => {
                        self.msg.pop().unwrap();
                    }
                    _ => (),
                };
                return true;
            }
        } else if let Event::Key(e) = ev {
            if let keys::OPEN_COMMIT = e {
                if !self.stage_empty {
                    self.show();
                    return true;
                }
            }
        }
        false
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn hide(&mut self) {
        self.visible = false
    }

    fn show(&mut self) {
        self.visible = true
    }
}

impl CommitComponent {
    ///
    pub fn new(queue: Queue) -> Self {
        Self {
            queue,
            msg: String::default(),
            stage_empty: true,
            visible: false,
        }
    }

    fn commit(&mut self) {
        if let HookResult::NotOk(e) =
            sync::hooks_commit_msg(CWD, &mut self.msg)
        {
            error!("commit-msg hook error: {}", e);
            self.queue.borrow_mut().push_back(
                InternalEvent::ShowMsg(format!(
                    "commit-msg hook error:\n{}",
                    e
                )),
            );
            return;
        }

        sync::commit(CWD, &self.msg);
        if let HookResult::NotOk(e) = sync::hooks_post_commit(CWD) {
            error!("post-commit hook error: {}", e);
            self.queue.borrow_mut().push_back(
                InternalEvent::ShowMsg(format!(
                    "post-commit hook error:\n{}",
                    e
                )),
            );
        }

        self.msg.clear();
        self.hide();

        self.queue
            .borrow_mut()
            .push_back(InternalEvent::Update(NeedsUpdate::ALL));
    }

    fn can_commit(&self) -> bool {
        !self.msg.is_empty()
    }

    ///
    pub fn set_stage_empty(&mut self, empty: bool) {
        self.stage_empty = empty;
    }
}
