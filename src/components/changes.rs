use super::{CommandBlocking, DrawableComponent};
use crate::{
    components::{CommandInfo, Component},
    keys,
    queue::{InternalEvent, NeedsUpdate, Queue},
    strings, ui,
};
use asyncgit::{hash, sync, StatusItem, StatusItemType, CWD};
use crossterm::event::Event;
use std::{
    borrow::Cow,
    cmp,
    convert::{From, TryFrom},
    path::Path,
};
use strings::commands;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Text,
    Frame,
};

///
pub struct ChangesComponent {
    title: String,
    items: Vec<StatusItem>,
    selection: Option<usize>,
    focused: bool,
    show_selection: bool,
    is_working_dir: bool,
    queue: Queue,
}

impl ChangesComponent {
    ///
    pub fn new(
        title: &str,
        focus: bool,
        is_working_dir: bool,
        queue: Queue,
    ) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),

            selection: None,
            focused: focus,
            show_selection: focus,
            is_working_dir,
            queue,
        }
    }

    ///
    pub fn update(&mut self, list: &[StatusItem]) {
        if hash(&self.items) != hash(list) {
            self.items = list.to_owned();

            let old_selection = self.selection.unwrap_or_default();
            self.selection = if self.items.is_empty() {
                None
            } else {
                Some(cmp::min(old_selection, self.items.len() - 1))
            };
        }
    }

    ///
    pub fn selection(&self) -> Option<StatusItem> {
        match self.selection {
            None => None,
            Some(i) => Some(self.items[i].clone()),
        }
    }

    ///
    pub fn focus_select(&mut self, focus: bool) {
        self.focus(focus);
        self.show_selection = focus;
    }

    ///
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn move_selection(&mut self, delta: i32) -> bool {
        let items_len = self.items.len();
        if items_len > 0 {
            if let Some(i) = self.selection {
                if let Ok(mut i) = i32::try_from(i) {
                    if let Ok(max) = i32::try_from(items_len) {
                        i = cmp::min(i + delta, max - 1);
                        i = cmp::max(i, 0);

                        if let Ok(i) = usize::try_from(i) {
                            self.selection = Some(i);
                            self.queue.borrow_mut().push_back(
                                InternalEvent::Update(
                                    NeedsUpdate::DIFF,
                                ),
                            );
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn index_add_remove(&mut self) -> bool {
        if let Some(i) = self.selection() {
            if self.is_working_dir {
                let path = Path::new(i.path.as_str());

                return sync::stage_add(CWD, path);
            } else {
                let path = Path::new(i.path.as_str());

                return sync::reset_stage(CWD, path);
            }
        }

        false
    }

    fn dispatch_reset_workdir(&mut self) -> bool {
        if let Some(i) = self.selection() {
            self.queue
                .borrow_mut()
                .push_back(InternalEvent::ConfirmResetFile(i.path));

            return true;
        }
        false
    }
}

impl DrawableComponent for ChangesComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, r: Rect) {
        let item_to_text = |idx: usize, i: &StatusItem| -> Text {
            let selected = self.show_selection
                && self.selection.map_or(false, |e| e == idx);
            let txt = if selected {
                format!("> {}", i.path)
            } else {
                format!("  {}", i.path)
            };
            let mut style = Style::default().fg(
                match i.status.unwrap_or(StatusItemType::Modified) {
                    StatusItemType::Modified => Color::LightYellow,
                    StatusItemType::New => Color::LightGreen,
                    StatusItemType::Deleted => Color::LightRed,
                    _ => Color::White,
                },
            );
            if selected {
                style = style.modifier(Modifier::BOLD); //.fg(Color::White);
            }

            Text::Styled(Cow::from(txt), style)
        };

        ui::draw_list(
            f,
            r,
            &self.title.to_string(),
            self.items
                .iter()
                .enumerate()
                .map(|(idx, e)| item_to_text(idx, e)),
            if self.show_selection {
                self.selection
            } else {
                None
            },
            self.focused,
        );
    }
}

impl Component for ChangesComponent {
    fn commands(
        &self,
        out: &mut Vec<CommandInfo>,
        _force_all: bool,
    ) -> CommandBlocking {
        let some_selection = self.selection().is_some();
        if self.is_working_dir {
            out.push(CommandInfo::new(
                commands::STAGE_FILE,
                some_selection,
                self.focused,
            ));
            out.push(CommandInfo::new(
                commands::RESET_FILE,
                some_selection,
                self.focused,
            ));
        } else {
            out.push(CommandInfo::new(
                commands::UNSTAGE_FILE,
                some_selection,
                self.focused,
            ));
        }

        out.push(CommandInfo::new(
            commands::SCROLL,
            self.items.len() > 1,
            self.focused,
        ));

        CommandBlocking::PassingOn
    }

    fn event(&mut self, ev: Event) -> bool {
        if self.focused {
            if let Event::Key(e) = ev {
                return match e {
                    keys::STATUS_STAGE_FILE => {
                        if self.index_add_remove() {
                            self.queue.borrow_mut().push_back(
                                InternalEvent::Update(
                                    NeedsUpdate::ALL,
                                ),
                            );
                        }
                        true
                    }
                    keys::STATUS_RESET_FILE => {
                        self.is_working_dir
                            && self.dispatch_reset_workdir()
                    }
                    keys::MOVE_DOWN => self.move_selection(1),
                    keys::MOVE_UP => self.move_selection(-1),
                    _ => false,
                };
            }
        }

        false
    }

    fn focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self, focus: bool) {
        self.focused = focus
    }
}
