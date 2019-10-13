// Extra sanity check for fat pointer layouts.

fn check_fat_pointer_layout() {
    use std::ptr::{read, write};
    {
        let r = &42;
        let thin = r as *const i32;
        let mut fat = r as &dyn std::fmt::Display as *const dyn std::fmt::Display;
        assert_eq!(
            unsafe { read(&fat as *const *const dyn std::fmt::Display as *const *const u8) },
            unsafe { read(&thin as *const *const i32 as *const *const u8) }
        );
        let r2 = &84;
        unsafe {
            write(
                &mut fat as *mut *const dyn std::fmt::Display as *mut *const u8,
                r2 as *const i32 as *const u8,
            );
        }
        assert_eq!(format!("{}", unsafe { &*fat }), "84");
    }
}

fn main() {
    check_fat_pointer_layout();
}
