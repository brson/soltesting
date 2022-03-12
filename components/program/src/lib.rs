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
//#[link_section = ".easy-decompile"]
fn powi_intr(f: f64, n: i32) -> f64 {
    // logical right shift before call
    unsafe { std::intrinsics::powif64(f, n) }
}

// works
#[no_mangle]
#[inline(never)]
//#[link_section = ".easy-decompile"]
fn powi_rt(f: f64, n: i32) -> f64 {
    // arithmetic right shift before call
    unsafe { __powidf2(f, n) }
}

#[no_mangle]
#[inline(never)]
//#[link_section = ".easy-decompile"]
fn c_fn(f: f64, n: i32) -> f64 {
    // arithmetic right shift before call
    unsafe { __powidf2(f, n) }
}

// prevent constant folding if powi is inlined into run_test
#[no_mangle]
#[inline(never)]
fn values() -> (f64, i32) {
    (10.0, -1)
}

#[no_mangle]
#[inline(never)]
fn run_test() {
    let (f, i) = values();
    let value = powi_rt(f, i);
    fmtprint(value)
}

#[no_mangle]
#[inline(never)]
fn fmtprint(value: f64) {
    let value = format!("{}", value);
    unsafe { sol_log_(value.as_ptr(), value.len()) };
}
