use libc::{c_char, c_void, size_t};
use std::sync::mpsc;

use super::*;

#[derive(Debug)]
pub enum ScreenEvent {
    AltScreen(AltScreenEvent),
    Bell,
    CursorBlink(CursorBlinkEvent),
    CursorShape(CursorShapeEvent),
    CursorVisible(CursorVisibleEvent),
    Damage(DamageEvent),
    IconName(IconNameEvent),
    Mouse(MouseEvent),
    MoveCursor(MoveCursorEvent),
    MoveRect(MoveRectEvent),
    Resize(ResizeEvent),
    Reverse(ReverseEvent),
    SbPopLine(SbPopLineEvent),
    SbPushLine(SbPushLineEvent),
    Title(TitleEvent),
}

#[derive(Debug)]
pub struct ScreenCallbacksConfig {
    pub damage: bool,
    pub move_rect: bool,
    pub move_cursor: bool,
    pub set_term_prop: bool,
    pub bell: bool,
    pub resize: bool,
    pub sb_pushline: bool,
    pub sb_popline: bool,
}

impl ScreenCallbacksConfig {
    pub fn all() -> ScreenCallbacksConfig {
        ScreenCallbacksConfig {
            damage: true,
            move_rect: true,
            move_cursor: true,
            set_term_prop: true,
            bell: true,
            resize: true,
            sb_pushline: true,
            sb_popline: true,
        }
    }

    pub fn none() -> ScreenCallbacksConfig {
        ScreenCallbacksConfig {
            damage: false,
            move_rect: false,
            move_cursor: false,
            set_term_prop: false,
            bell: false,
            resize: false,
            sb_pushline: false,
            sb_popline: false,
        }
    }
}

pub enum DamageSize {
    Cell,   // every cell
    Row,    // entire rows
    Screen, // entire screen
    Scroll, // entire screen + scrollrect
}

impl VTerm {
    /// Reset the screen. I've observed this needs to happen before using or segfaults will occur.
    pub fn screen_reset(&mut self, is_hard: bool) {
        unsafe { ffi::vterm_screen_reset(self.screen_ptr.as_mut(), super::bool_to_int(is_hard)) }
    }

    /// Return the cell at the given position
    pub fn screen_get_cell(&self, pos: &Pos) -> ScreenCell {
        let size = self.get_size();
        if pos.x >= size.width || pos.y >= size.height {
            panic!(
                "given position out of bounds: size={:?} pos={:?}",
                size, pos
            );
        }

        let cell_buf = unsafe { ffi::vterm_cell_new() };
        unsafe {
            ffi::vterm_screen_get_cell(
                self.screen_ptr.as_ref(),
                ffi::VTermPos::from_pos(&pos),
                cell_buf,
            )
        };
        let cell = ScreenCell::from_ptr(cell_buf, &self); // shouldn't this take &cell_buf?
        unsafe { ffi::vterm_cell_free(cell_buf) };

        cell
    }

    // Returns the text within the rect as a String. Invalid utf8 sequences are replaces with or
    // panics if invalid utf8 bytes are found
    pub fn screen_get_text_lossy(&self, rect: &Rect) -> String {
        let bytes = self.get_text_as_bytes(rect);
        String::from_utf8_lossy(&bytes).into_owned()
    }

    // Returns the text within the rect as a String or panics if invalid utf8 bytes are found
    pub fn screen_get_text(&self, rect: &Rect) -> Result<String, ::std::string::FromUtf8Error> {
        let bytes = self.get_text_as_bytes(rect);
        let v = String::from_utf8(bytes)?;
        Ok(v)
    }

    fn get_text_as_bytes(&self, rect: &Rect) -> Vec<u8> {
        let screen_rect = Rect::new(Pos::new(0, 0), self.get_size());
        if !screen_rect.contains_rect(&rect) {
            panic!(
                "given rect out of bounds: size={:?} rect={:?}",
                self.get_size(),
                rect
            );
        }

        let size: usize = rect.size.width * rect.size.height * ffi::VTERM_MAX_CHARS_PER_CELL;
        let mut bytes = Vec::with_capacity(size);
        unsafe { bytes.set_len(size) };
        let bytes_ptr: *mut c_char = (&mut bytes[0..size]).as_mut_ptr();

        unsafe {
            let len = ffi::vterm_screen_get_text(
                self.screen_ptr.as_ref(),
                bytes_ptr,
                size as size_t,
                ffi::VTermRect::from_rect(&rect),
            );
            bytes.set_len(len);
        }

        bytes.into_iter().map(|c| c as u8).collect()
    }

    pub fn screen_flush_damage(&mut self) {
        unsafe { ffi::vterm_screen_flush_damage(self.screen_ptr.as_mut()) };
    }

    pub fn screen_set_damage_merge(&mut self, size: DamageSize) {
        let ffi_size = match size {
            DamageSize::Cell => ffi::VTermDamageSize::VTermDamageCell,
            DamageSize::Row => ffi::VTermDamageSize::VTermDamageRow,
            DamageSize::Screen => ffi::VTermDamageSize::VTermDamageScreen,
            DamageSize::Scroll => ffi::VTermDamageSize::VTermDamageScroll,
        };
        unsafe { ffi::vterm_screen_set_damage_merge(self.screen_ptr.as_mut(), ffi_size) };
    }

    pub fn screen_get_cells_in_rect(&self, rect: &Rect) -> Vec<ScreenCell> {
        let mut cells: Vec<ScreenCell> = Vec::new(); // capacity is known here FYI

        for pos in rect.positions() {
            cells.push(self.screen_get_cell(&pos));
        }

        cells
    }

    /// calling this method will setup the vterm to generate ScreenEvent messages to a channel. The
    /// returned result indicates whether the channel was already created. The receiver end of the
    /// channel can be had by accessing the screen_events_rx field.
    pub fn screen_receive_events(&mut self, config: &ScreenCallbacksConfig) {
        let mut callbacks: ffi::VTermScreenCallbacks = Default::default();

        callbacks.damage = if config.damage {
            Some(screen_callbacks::damage)
        } else {
            None
        };
        callbacks.move_rect = if config.move_rect {
            Some(screen_callbacks::move_rect)
        } else {
            None
        };
        callbacks.move_cursor = if config.move_cursor {
            Some(screen_callbacks::move_cursor)
        } else {
            None
        };
        callbacks.set_term_prop = if config.set_term_prop {
            Some(screen_callbacks::set_term_prop)
        } else {
            None
        };
        callbacks.bell = if config.bell {
            Some(screen_callbacks::bell)
        } else {
            None
        };
        callbacks.resize = if config.resize {
            Some(screen_callbacks::resize)
        } else {
            None
        };
        callbacks.sb_pushline = if config.sb_pushline {
            Some(screen_callbacks::sb_pushline)
        } else {
            None
        };
        callbacks.sb_popline = if config.sb_popline {
            Some(screen_callbacks::sb_popline)
        } else {
            None
        };

        self.screen_callbacks = Some(callbacks);

        if self.screen_event_tx.is_none() {
            let (tx, rx) = mpsc::channel();
            self.screen_event_tx = Some(tx);
            self.screen_event_rx = Some(rx);
        }

        unsafe {
            let self_ptr: *mut c_void = self as *mut _ as *mut c_void;
            ffi::vterm_screen_set_callbacks(
                self.screen_ptr.as_mut(),
                self.screen_callbacks.as_ref().unwrap(),
                self_ptr,
            );
        }
    }
}

mod tests {
    #![allow(unused_imports)]
    use super::super::*;

    #[test]
    fn screen_can_reset() {
        let mut vterm: VTerm = VTerm::new(&Size {
            height: 2,
            width: 2,
        })
        .unwrap();
        vterm.screen_reset(true);
    }
}
