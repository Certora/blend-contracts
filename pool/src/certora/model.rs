static mut CHECKED: bool = false;
static mut SKOLEM_I: u32 = 0;

pub fn get_checked() -> bool {
    unsafe { CHECKED }
}
pub fn set_checked() {
    unsafe { CHECKED = true }
}

pub fn skolem_i() -> u32 {
    unsafe { SKOLEM_I }
}
pub fn init() {
    unsafe {
        SKOLEM_I = nondet::nondet();
        CHECKED = nondet::nondet();
    }
}
