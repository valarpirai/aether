//! Procedural macro for #[aether_export]
//!
//! This macro generates FFI wrapper code for Aether plugin functions.
//! Supports both V1 protocol (i64 only) and V2 protocol (String, Vec, HashMap).

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Type, TypePath};

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
///
/// #[aether_export]
/// fn greet(name: String) -> String {
///     format!("Hello, {}!", name)
/// }
/// ```
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

    // Check if this is a V1 (i64-only) or V2 (complex types) function
    let uses_v1_protocol = params.iter().all(|(_, ty)| is_i64_type(ty))
        && matches!(&input_fn.sig.output, ReturnType::Type(_, ty) if is_i64_type(ty));

    let original_fn = &input_fn;

    if uses_v1_protocol {
        // Generate V1 protocol wrapper (backward compatible)
        let param_extractions = params.iter().enumerate().map(|(i, (name, ty))| {
            quote! {
                let #name: #ty = unsafe { *args.offset(#i as isize) };
            }
        });

        let param_names: Vec<_> = params.iter().map(|(name, _)| name).collect();

        let expanded = quote! {
            #original_fn

            #[doc(hidden)]
            pub unsafe extern "C" fn #ffi_fn_name(args: *const i64, argc: ::std::ffi::c_int) -> i64 {
                if argc != #param_count as ::std::ffi::c_int {
                    return i64::MIN; // Signal error
                }

                #(#param_extractions)*

                #fn_name(#(#param_names),*)
            }
        };

        TokenStream::from(expanded)
    } else {
        // Generate V2 protocol wrapper (complex types)
        let param_conversions = params.iter().enumerate().map(|(i, (name, ty))| {
            quote! {
                let #name: #ty = match aether_plugin::FromAether::from_aether(*args.offset(#i as isize)) {
                    Ok(v) => v,
                    Err(e) => {
                        let err_msg = ::std::ffi::CString::new(e).unwrap();
                        *out_error = aether_plugin::convert::aether_value_new_string(err_msg.as_ptr());
                        return ::std::ptr::null();
                    }
                };
            }
        });

        let param_names: Vec<_> = params.iter().map(|(name, _)| name).collect();

        // Functions returning Result<T, E> route Err through out_error so the
        // failure surfaces to Aether as a catchable plugin error rather than a
        // silent null. Other return types convert directly.
        let call_and_return = if returns_result(&input_fn.sig.output) {
            quote! {
                match #fn_name(#(#param_names),*) {
                    Ok(v) => aether_plugin::ToAether::to_aether(v),
                    Err(e) => {
                        let err_msg = ::std::ffi::CString::new(e.to_string())
                            .unwrap_or_else(|_| ::std::ffi::CString::new("plugin error").unwrap());
                        *out_error = aether_plugin::convert::aether_value_new_string(err_msg.as_ptr());
                        ::std::ptr::null()
                    }
                }
            }
        } else {
            quote! {
                let result = #fn_name(#(#param_names),*);
                aether_plugin::ToAether::to_aether(result)
            }
        };

        let expanded = quote! {
            #original_fn

            #[doc(hidden)]
            pub unsafe extern "C" fn #ffi_fn_name(
                args: *const aether_plugin::AetherValuePtr,
                argc: ::std::ffi::c_int,
                out_error: *mut aether_plugin::AetherValuePtr
            ) -> aether_plugin::AetherValuePtr {
                if argc != #param_count as ::std::ffi::c_int {
                    let err_msg = ::std::ffi::CString::new(format!("Expected {} arguments, got {}", #param_count, argc)).unwrap();
                    *out_error = aether_plugin::convert::aether_value_new_string(err_msg.as_ptr());
                    return ::std::ptr::null();
                }

                #(#param_conversions)*

                #call_and_return
            }
        };

        TokenStream::from(expanded)
    }
}

/// Check if a return type is `Result<...>` (by outermost path segment).
fn returns_result(output: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = output {
        if let Type::Path(TypePath { path, .. }) = &**ty {
            if let Some(seg) = path.segments.last() {
                return seg.ident == "Result";
            }
        }
    }
    false
}

/// Check if a type is i64
fn is_i64_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.is_ident("i64")
    } else {
        false
    }
}
