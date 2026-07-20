//! Procedural macro for #[aether_export]
//!
//! This macro generates FFI wrapper code for Aether plugin functions.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Type};

/// Marks a function for export to Aether
///
/// # Example
///
/// ```rust
/// use aether_plugin::*;
///
/// #[aether_export]
/// fn add(a: i64, b: i64) -> i64 {
///     a + b
/// }
/// ```
///
/// This generates:
/// - An FFI wrapper function
/// - Type conversion boilerplate
/// - Error handling
/// - Registration metadata
#[proc_macro_attribute]
pub fn aether_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let ffi_fn_name = syn::Ident::new(&format!("{}_ffi", fn_name), fn_name.span());

    // Extract parameters
    let params: Vec<_> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    Some((pat_ident.ident.clone(), pat_type.ty.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let param_count = params.len();

    // Generate parameter extraction
    let param_extractions = params.iter().enumerate().map(|(i, (name, ty))| {
        quote! {
            let #name: #ty = unsafe { *args.offset(#i as isize) };
        }
    });

    // Generate function call
    let param_names: Vec<_> = params.iter().map(|(name, _)| name).collect();
    let original_fn = &input_fn;

    // Check return type
    let returns_i64 = matches!(
        &input_fn.sig.output,
        ReturnType::Type(_, ty) if matches!(&**ty, Type::Path(p) if p.path.is_ident("i64"))
    );

    let expanded = if returns_i64 {
        quote! {
            #original_fn

            #[doc(hidden)]
            unsafe extern "C" fn #ffi_fn_name(args: *const i64, argc: ::std::ffi::c_int) -> i64 {
                if argc != #param_count as ::std::ffi::c_int {
                    return i64::MIN; // Signal error
                }

                #(#param_extractions)*

                #fn_name(#(#param_names),*)
            }
        }
    } else {
        // For now, only support i64 return
        return syn::Error::new_spanned(
            &input_fn.sig.output,
            "Only i64 return type is supported in MVP",
        )
        .to_compile_error()
        .into();
    };

    TokenStream::from(expanded)
}
