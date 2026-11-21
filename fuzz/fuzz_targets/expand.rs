//! Fuzzes the derive expansion: any input must expand without panicking into parsable items.

#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;

/// The derive's expansion logic, shared with the proc-macro crate.
///
/// shortcut: a proc-macro crate cannot be linked as a library, so the module is included by
/// path; split it into its own crate if the fuzz target ever needs more than this one file.
#[path = "../../crates/uniform-array-derive/src/expand.rs"]
mod expand;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(tokens) = source.parse::<proc_macro2::TokenStream>() else {
        return;
    };

    let output = expand::expand(tokens);

    assert!(
        syn::parse2::<syn::File>(output).is_ok(),
        "expansion does not parse for input: {source}"
    );
});
