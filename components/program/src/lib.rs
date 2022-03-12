#![allow(unused)]
#![feature(core_intrinsics)]

solana_program::custom_heap_default!();
solana_program::custom_panic_default!();

extern "C" {
    fn sol_log_(ptr: *const u8, len: usize);
}

// also happens with f32
use solana_program::msg;

#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    run_test();
    0
}

extern "C" {
    fn __powidf2(f: f64, n: i32) -> f64;
}

// does not work
#[no_mangle]
#[inline(never)]
fn powi_intr(f: f64, n: i32) -> f64 {
    unsafe { std::intrinsics::powif64(f, n) }
    //unsafe { __powidf2(f, n) }
}

// works
#[no_mangle]
#[inline(never)]
fn powi_rt(f: f64, n: i32) -> f64 {
    unsafe { __powidf2(f, n) }
}

#[no_mangle]
#[inline(never)]
fn run_test() {
    let value = powi_intr(10.0, -1);
    fmtprint(value)
}

#[no_mangle]
#[inline(never)]
fn fmtprint(value: f64) {
    let value = format!("{}", value);
    unsafe { sol_log_(value.as_ptr(), value.len()) };
}
