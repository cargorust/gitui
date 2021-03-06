use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const fn no_mod(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
    }
}

pub const FOCUS_WORKDIR: KeyEvent = no_mod(KeyCode::Char('1'));
pub const FOCUS_STAGE: KeyEvent = no_mod(KeyCode::Char('2'));
pub const FOCUS_RIGHT: KeyEvent = no_mod(KeyCode::Right);
pub const FOCUS_LEFT: KeyEvent = no_mod(KeyCode::Left);
pub const STATUS_RESET_FILE: KeyEvent = no_mod(KeyCode::Char('D'));
pub const STATUS_STAGE_FILE: KeyEvent = no_mod(KeyCode::Enter);
pub const EXIT_1: KeyEvent = no_mod(KeyCode::Esc);
pub const EXIT_POPUP: KeyEvent = no_mod(KeyCode::Esc);
pub const EXIT_2: KeyEvent = no_mod(KeyCode::Char('q'));
pub const CLOSE_MSG: KeyEvent = no_mod(KeyCode::Enter);
pub const OPEN_COMMIT: KeyEvent = no_mod(KeyCode::Char('c'));
pub const OPEN_HELP: KeyEvent = no_mod(KeyCode::Char('h'));
pub const MOVE_UP: KeyEvent = no_mod(KeyCode::Up);
pub const MOVE_DOWN: KeyEvent = no_mod(KeyCode::Down);
