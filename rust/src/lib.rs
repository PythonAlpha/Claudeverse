//! Profile search scorer, compiled to WebAssembly.
//!
//! Job: the page asks "how well does this piece of text match what the
//! visitor typed?" and Rust answers with a number. 0 means no match.
//!
//! No dependencies and no allocator: JavaScript writes text into two fixed
//! buffers that live inside the WASM module, then calls `score`.

#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

const CAP: usize = 4096;

static mut QUERY: [u8; CAP] = [0; CAP];
static mut TEXT: [u8; CAP] = [0; CAP];

#[no_mangle]
pub extern "C" fn capacity() -> usize {
    CAP
}

#[no_mangle]
pub extern "C" fn query_ptr() -> *mut u8 {
    core::ptr::addr_of_mut!(QUERY) as *mut u8
}

#[no_mangle]
pub extern "C" fn text_ptr() -> *mut u8 {
    core::ptr::addr_of_mut!(TEXT) as *mut u8
}

#[inline]
fn lower(b: u8) -> u8 {
    if b.is_ascii_uppercase() { b + 32 } else { b }
}

#[inline]
fn is_space(b: u8) -> bool {
    b == b' ' || b == b'\n' || b == b'\t'
}

#[inline]
unsafe fn at(base: *const u8, i: usize) -> u8 {
    lower(*base.add(i))
}

/// Find `needle` (len `n`, starting at `q + qs`) inside the text.
/// Returns the position of the first match, or `usize::MAX` if none.
unsafe fn find(q: *const u8, qs: usize, n: usize, t: *const u8, tlen: usize) -> usize {
    if n == 0 || n > tlen {
        return usize::MAX;
    }
    let mut i = 0;
    while i + n <= tlen {
        let mut j = 0;
        while j < n && at(t, i + j) == at(q, qs + j) {
            j += 1;
        }
        if j == n {
            return i;
        }
        i += 1;
    }
    usize::MAX
}

/// Score the text in TEXT against the query in QUERY.
///
/// - Every word in the query must appear in the text, otherwise 0.
/// - Each matched word earns 10 points.
/// - A word that starts a word in the text earns 5 more.
/// - An empty query returns 1, so everything stays visible.
#[no_mangle]
pub extern "C" fn score(qlen: usize, tlen: usize) -> u32 {
    let q = query_ptr() as *const u8;
    let t = text_ptr() as *const u8;
    let qlen = if qlen > CAP { CAP } else { qlen };
    let tlen = if tlen > CAP { CAP } else { tlen };

    let mut total: u32 = 0;
    let mut words: u32 = 0;
    let mut i = 0;

    unsafe {
        while i < qlen {
            while i < qlen && is_space(*q.add(i)) {
                i += 1;
            }
            let start = i;
            while i < qlen && !is_space(*q.add(i)) {
                i += 1;
            }
            let n = i - start;
            if n == 0 {
                continue;
            }
            words += 1;

            let pos = find(q, start, n, t, tlen);
            if pos == usize::MAX {
                return 0;
            }
            total += 10;
            if pos == 0 || is_space(*t.add(pos - 1)) {
                total += 5;
            }
        }
    }

    if words == 0 { 1 } else { total }
}
