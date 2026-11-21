//! Unit and property tests for [`expand`](super::expand), run on the generated token trees.
//!
//! Type equality is checked by rustc on the expanded code, so it is covered by the
//! `compile_fail` doctests in the `example` crate, not here.

use super::expand;
use proptest::prelude::*;

/// Field types the generated structs pick from.
const TYPES: &[&str] = &[
    "f32",
    "u8",
    "T",
    "Box<u8>",
    "[f32; 3]",
    "&'a str",
    "core::primitive::f32",
    "dyn core::fmt::Debug + Send",
];

/// `repr` attributes the generated structs pick from, with whether the slice casts accept them.
const REPRS: &[(&str, bool)] = &[
    ("", false),
    ("#[repr(C)]", true),
    ("#[repr(transparent)]", true),
    ("#[repr(Rust)]", false),
    ("#[repr(C, align(16))]", false),
    ("#[repr(C, packed)]", false),
    ("#[repr(align(8), C)]", false),
    ("#[repr(C)] #[repr(Rust)]", true),
];

/// Expands `source` and parses the result as a file, failing the test if it does not parse.
fn expand_to_file(source: &str) -> syn::File {
    let tokens: proc_macro2::TokenStream = source.parse().expect("test input must tokenize");
    let output = expand(tokens);

    syn::parse2(output.clone())
        .unwrap_or_else(|e| panic!("expansion does not parse ({e}): {output}"))
}

/// Builds a struct named `S` with the given attributes and field types.
fn make_struct(attrs: &str, types: &[&str], tuple: bool) -> String {
    if tuple {
        format!("{attrs} pub struct S<'a, T>({});", types.join(", "))
    } else {
        let fields: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("f{i}: {ty}"))
            .collect();
        format!("{attrs} pub struct S<'a, T> {{ {} }}", fields.join(", "))
    }
}

/// Returns whether the expansion contains a `compile_error!` invocation.
fn has_compile_error(file: &syn::File) -> bool {
    file.items.iter().any(|item| match item {
        syn::Item::Macro(m) => m
            .mac
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "compile_error"),
        _ => false,
    })
}

/// Returns the impl blocks of `file` for the trait named `trait_name`, or inherent impls for `None`.
fn impls<'f>(
    file: &'f syn::File,
    trait_name: Option<&'f str>,
) -> impl Iterator<Item = &'f syn::ItemImpl> {
    file.items.iter().filter_map(move |item| match item {
        syn::Item::Impl(imp) => {
            let name = imp
                .trait_
                .as_ref()
                .and_then(|(path, _)| path.segments.last())
                .map(|s| s.ident.to_string());
            (name.as_deref() == trait_name).then_some(imp)
        }
        _ => None,
    })
}

/// Returns the body of the method `name` in `imp`.
fn method<'i>(imp: &'i syn::ItemImpl, name: &str) -> Option<&'i syn::Block> {
    imp.items.iter().find_map(|item| match item {
        syn::ImplItem::Fn(f) if f.sig.ident == name => Some(&f.block),
        _ => None,
    })
}

/// Returns the boolean literal that the generated `is_empty` method returns.
fn is_empty_value(file: &syn::File) -> Option<bool> {
    let body = impls(file, None).find_map(|imp| method(imp, "is_empty"))?;

    match body.stmts.last()? {
        syn::Stmt::Expr(
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(b),
                ..
            }),
            None,
        ) => Some(b.value),
        _ => None,
    }
}

/// Returns the number of arms in the `match` of `Index::index`, including the panic arm.
fn index_arm_count(file: &syn::File) -> Option<usize> {
    let body = impls(file, Some("Index")).find_map(|imp| method(imp, "index"))?;

    match body.stmts.last()? {
        syn::Stmt::Expr(syn::Expr::Match(m), None) => Some(m.arms.len()),
        _ => None,
    }
}

/// Returns the integer literal that the generated `len` method returns.
fn len_value(file: &syn::File) -> Option<usize> {
    let body = impls(file, None).find_map(|imp| method(imp, "len"))?;

    match body.stmts.last()? {
        syn::Stmt::Expr(
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            }),
            None,
        ) => n.base10_parse().ok(),
        _ => None,
    }
}

/// A field type with a bare `+` still expands to parsable code (found by fuzzing).
#[test]
fn trait_object_field_parses() {
    let file = expand_to_file("struct S(dyn core::fmt::Debug + Send);");

    assert_eq!(index_arm_count(&file), Some(2));
}

/// Enums are reported as a compile error, not a panic.
#[test]
fn enum_becomes_compile_error() {
    let file = expand_to_file("#[derive(UniformArray)] enum E { A }");

    assert!(has_compile_error(&file));
}

/// Input that is not a struct definition is reported as a compile error.
#[test]
fn garbage_becomes_compile_error() {
    let file = expand_to_file("fn not_a_struct() {}");

    assert!(has_compile_error(&file));
}

/// A struct without fields gets `len` and `is_empty` only.
#[test]
fn unit_struct_gets_only_len() {
    let file = expand_to_file("pub struct Unit;");

    assert_eq!(len_value(&file), Some(0));
    assert_eq!(index_arm_count(&file), None);
    assert!(!has_compile_error(&file));
}

/// Without `safety_gate`, the slice conversions are gated on `feature = "unsafe"`.
#[test]
fn default_gate_is_unsafe() {
    let output = expand("#[repr(C)] struct S(u8, u8);".parse().unwrap()).to_string();

    assert!(output.contains(r#"feature = "unsafe""#), "{output}");
}

/// `safety_gate` replaces the default gate, and `docs_rs` emits a valid `cfg_attr`.
#[test]
fn custom_gate_and_docs_rs() {
    let source = r#"#[uniform_array(safety_gate = "fast", docs_rs = "docsrs")] #[repr(C)] struct S(u8, u8);"#;
    let output = expand(source.parse().unwrap()).to_string();

    assert!(output.contains(r#"feature = "fast""#), "{output}");
    assert!(output.contains("cfg_attr (docsrs"), "{output}");
    assert!(!output.contains(r#"feature = "unsafe""#), "{output}");
}

/// The layout error only fires with the safety gate enabled.
#[test]
fn layout_error_is_gated() {
    let output = expand("struct S(u8, u8);".parse().unwrap()).to_string();
    let gated = r#"# [cfg (feature = "unsafe")] :: core :: compile_error !"#;

    assert!(output.contains(gated), "{output}");
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: if cfg!(miri) { 4 } else { 256 },
        failure_persistence: if cfg!(miri) { None } else { ProptestConfig::default().failure_persistence },
        ..ProptestConfig::default()
    })]

    /// The expansion always parses, has one `Index` arm per field plus the panic arm,
    /// reports the field count from `len` and `is_empty`, and rejects exactly the unsupported layouts.
    #[test]
    fn expansion_matches_struct(
        types in prop::collection::vec(prop::sample::select(TYPES), 0..6),
        (repr, layout_ok) in prop::sample::select(REPRS),
        tuple in any::<bool>(),
    ) {
        let file = expand_to_file(&make_struct(repr, &types, tuple));

        prop_assert_eq!(len_value(&file), Some(types.len()));
        prop_assert_eq!(is_empty_value(&file), Some(types.is_empty()));

        if types.is_empty() {
            prop_assert_eq!(index_arm_count(&file), None);
            prop_assert!(!has_compile_error(&file));
        } else {
            prop_assert_eq!(index_arm_count(&file), Some(types.len() + 1));
            prop_assert_eq!(has_compile_error(&file), !layout_ok);
        }
    }

    /// Arbitrary token input never panics and always yields parsable items.
    #[test]
    fn arbitrary_input_never_panics(source in "[a-zA-Z0-9_ ,;:<>(){}#\\[\\]'&=\"]{0,64}") {
        if let Ok(tokens) = source.parse::<proc_macro2::TokenStream>() {
            prop_assert!(syn::parse2::<syn::File>(expand(tokens)).is_ok());
        }
    }
}
