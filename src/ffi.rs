extern crate libc;

use libc::{c_int};

pub enum VTerm {}
pub enum VTermScreen {}

extern {
    pub fn vterm_new(rows: c_int, cols: c_int) -> *mut VTerm;
    pub fn vterm_free(vterm: *mut VTerm);
    pub fn vterm_get_size(vterm: *const VTerm, rowsp: *mut c_int, colsp: *mut c_int);
    pub fn vterm_set_size(vterm: *mut VTerm, rows: c_int, cols: c_int);
    pub fn vterm_get_utf8(vterm: *const VTerm) -> c_int;
    pub fn vterm_set_utf8(vterm: *mut VTerm, is_utf8: c_int);
    pub fn vterm_obtain_screen(vterm: *mut VTerm) -> *mut VTermScreen;

    pub fn vterm_screen_reset(screen: *mut VTermScreen, hard: c_int);
}

mod tests {
    extern crate libc;

    use libc::{c_int};
    use super::*;

    #[test]
    fn vterm_can_create_and_destroy() {
        unsafe {
            let vterm_ptr: *mut VTerm = vterm_new(2, 2);
            vterm_free(vterm_ptr);
        }
    }

    #[test]
    fn vterm_can_get_size() {
        unsafe {
            let vterm_ptr: *mut VTerm = vterm_new(2, 2);
            let mut cols: c_int = 0;
            let mut rows: c_int = 0;
            vterm_get_size(vterm_ptr, &mut cols, &mut rows);
            assert_eq!((2, 2), (cols, rows));

            vterm_free(vterm_ptr);
        }
    }

    #[test]
    fn vterm_can_set_size() {
        unsafe {
            let vterm_ptr: *mut VTerm = vterm_new(2, 2);
            vterm_set_size(vterm_ptr, 1, 1);

            let mut cols: c_int = 0;
            let mut rows: c_int = 0;
            vterm_get_size(vterm_ptr, &mut cols, &mut rows);
            assert_eq!((1, 1), (cols, rows));

            vterm_free(vterm_ptr);
        }
    }

    #[test]
    fn vterm_can_get_and_set_utf8() {
        unsafe {
            let vterm_ptr: *mut VTerm = vterm_new(2, 2);

            vterm_set_utf8(vterm_ptr, 1);
            let val = vterm_get_utf8(vterm_ptr);
            assert_eq!(1, val); // not sure why this doesnt work

            vterm_free(vterm_ptr);
        }
    }

    #[test]
    fn vterm_can_obtain_screen() {
        unsafe {
            let vterm_ptr: *mut VTerm = vterm_new(2, 2);
            vterm_obtain_screen(vterm_ptr);
            vterm_free(vterm_ptr);
        }
    }

    #[test]
    fn screen_can_reset() {
        unsafe {
            let vterm_ptr: *mut VTerm = vterm_new(2, 2);
            let screen_ptr = vterm_obtain_screen(vterm_ptr);
            vterm_screen_reset(screen_ptr, 1);
            vterm_free(vterm_ptr);
        }
    }
}
