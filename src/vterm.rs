use libc::{c_int, c_void, size_t};
use std::sync::mpsc;
use std::ptr::Unique;

use super::*;

pub struct VTerm {
    pub ptr:              Unique<ffi::VTerm>,
    pub screen_event_tx:  Option<mpsc::Sender<ScreenEvent>>,
    pub screen_ptr:       Unique<ffi::VTermScreen>,
    pub state_ptr:        Unique<ffi::VTermState>,
}

impl VTerm {
    pub fn new(size: ScreenSize) -> VTerm {
        // TODO how to detect error?
        let mut vterm_ptr = unsafe { Unique::new(ffi::vterm_new(size.rows as c_int, size.cols as c_int)) };
        let screen_ptr = unsafe { Unique::new(ffi::vterm_obtain_screen(vterm_ptr.get_mut())) };
        let state_ptr = unsafe { Unique::new(ffi::vterm_obtain_state(vterm_ptr.get_mut())) };

        let mut vterm = VTerm {
            ptr: vterm_ptr,
            screen_event_tx: None,
            screen_ptr: screen_ptr,
            state_ptr: state_ptr,
        };

        vterm.screen_reset(true);

        vterm
    }

    pub fn get_size(&self) -> ScreenSize {
        let mut cols: c_int = 0;
        let mut rows: c_int = 0;
        unsafe { ffi::vterm_get_size(self.ptr.get(), &mut cols, &mut rows); }
        ScreenSize { rows: rows as u16, cols: cols as u16 }
    }

    pub fn set_size(&mut self, size: ScreenSize) {
        unsafe { ffi::vterm_set_size(self.ptr.get_mut(), size.cols as c_int, size.rows as c_int); }
    }

    pub fn get_utf8(&self) -> bool {
        unsafe { super::int_to_bool(ffi::vterm_get_utf8(self.ptr.get())) }
    }

    pub fn set_utf8(&mut self, is_utf8: bool) {
        unsafe { ffi::vterm_set_utf8(self.ptr.get_mut(), super::bool_to_int(is_utf8)) }
    }

    pub fn write(&mut self, input: &[u8]) -> u32 {
        unsafe {
            ffi::vterm_input_write(self.ptr.get_mut(), input.as_ptr(), input.len() as size_t) as u32
        }
    }

    /// Return a channel Receiver that will be sent ScreenEvents.
    pub fn receive_screen_events(&mut self) -> mpsc::Receiver<ScreenEvent> {
        let (tx, rx) = mpsc::channel();
        self.screen_event_tx = Some(tx);

        unsafe {
            let ptr: *mut c_void = self as *mut _ as *mut c_void;
            ffi::vterm_screen_set_callbacks(self.screen_ptr.get_mut(), &::screen::SCREEN_CALLBACKS, ptr);
        }

        rx
    }
}

impl Drop for VTerm {
    fn drop(&mut self) {
        unsafe { ffi::vterm_free(self.ptr.get_mut()) }
    }
}

mod tests {
    #[test]
    fn vterm_can_create_and_destroy() {
        let vterm: ::VTerm = ::VTerm::new(::ScreenSize { rows: 2, cols: 2 });
        drop(vterm);
    }

    #[test]
    fn vterm_can_get_size() {
        let vterm: ::VTerm = ::VTerm::new(::ScreenSize { rows: 2, cols: 2 });
        let size = vterm.get_size();
        assert_eq!((2, 2), (size.rows, size.cols));
    }

    #[test]
    fn vterm_can_set_size() {
        let mut vterm: ::VTerm = ::VTerm::new(::ScreenSize { rows: 2, cols: 2 });
        vterm.set_size(::ScreenSize { cols: 1, rows: 1 });
        let size = vterm.get_size();
        assert_eq!((1, 1), (size.rows, size.cols));
    }

    #[test]
    fn vterm_can_get_and_set_utf8() {
        let mut vterm: ::VTerm = ::VTerm::new(::ScreenSize { rows: 2, cols: 2 });
        vterm.set_utf8(true);
        assert_eq!(true, vterm.get_utf8());

        vterm.set_utf8(false);
        assert_eq!(false, vterm.get_utf8());
    }

    #[test]
    fn vterm_can_write() {
        let mut vterm: ::VTerm = ::VTerm::new(::ScreenSize { rows: 2, cols: 2 });
        let input: &[u8] = "abcd".as_bytes();
        assert_eq!(4, vterm.write(input));
    }
}
