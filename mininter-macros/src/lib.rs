use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Type};

/// Marks a Rust function as callable from MiniScript.
///
/// The macro generates:
/// - A native wrapper that converts `&[Value]` args to typed params
/// - A `{name}_api()` function returning an `ApiFunction` for VM registration
///
/// # Supported parameter types
/// Any type implementing `FromValue`: f64, bool, String, Value
///
/// # Supported return types
/// Any type implementing `IntoValue`: f64, bool, String, Value, ()
///
/// # Example
/// ```ignore
/// #[api]
/// fn sqrt(x: f64) -> f64 {
///     x.sqrt()
/// }
///
/// // Then register:
/// vm.register(sqrt_api());
/// ```
#[proc_macro_attribute]
pub fn api(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let wrapper_name = format_ident!("__{}_native", fn_name);
    let api_fn_name = format_ident!("{}_api", fn_name);

    // Check if this is a variadic function: single param `&[Value]`
    let is_variadic = is_variadic_sig(&input_fn);

    let wrapper_body = if is_variadic {
        // Variadic: pass args slice directly
        let has_return = !matches!(input_fn.sig.output, ReturnType::Default);
        if has_return {
            quote! {
                let __result = #fn_name(args);
                ::std::result::Result::Ok(
                    <_ as mininter::value::IntoValue>::into_value(__result)
                )
            }
        } else {
            quote! {
                #fn_name(args);
                ::std::result::Result::Ok(mininter::value::Value::Null)
            }
        }
    } else {
        // Fixed arity: extract and convert each param
        let params: Vec<_> = input_fn
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(ident) = &*pat_type.pat {
                        Some((ident.ident.clone(), &*pat_type.ty))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let arity = params.len();
        let extractions: Vec<_> = params
            .iter()
            .enumerate()
            .map(|(i, (name, ty))| {
                quote! {
                    let #name: #ty = <#ty as mininter::value::FromValue>::from_value(&args[#i])?;
                }
            })
            .collect();

        let param_names: Vec<_> = params.iter().map(|(name, _)| name).collect();

        let has_return = !matches!(input_fn.sig.output, ReturnType::Default);

        let call_and_return = if has_return {
            quote! {
                let __result = #fn_name(#(#param_names),*);
                ::std::result::Result::Ok(
                    <_ as mininter::value::IntoValue>::into_value(__result)
                )
            }
        } else {
            quote! {
                #fn_name(#(#param_names),*);
                ::std::result::Result::Ok(mininter::value::Value::Null)
            }
        };

        quote! {
            if args.len() != #arity {
                return ::std::result::Result::Err(
                    mininter::error::RuntimeError::WrongArgCount {
                        expected: #arity,
                        got: args.len(),
                    }
                );
            }
            #(#extractions)*
            #call_and_return
        }
    };

    let expanded = quote! {
        #input_fn

        #[allow(non_snake_case)]
        fn #wrapper_name(
            args: &[mininter::value::Value],
        ) -> ::std::result::Result<mininter::value::Value, mininter::error::RuntimeError> {
            #wrapper_body
        }

        /// Returns an [`ApiFunction`] that can be registered on a [`Vm`].
        pub fn #api_fn_name() -> mininter::vm::ApiFunction {
            mininter::vm::ApiFunction {
                name: #fn_name_str,
                func: #wrapper_name,
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_variadic_sig(func: &ItemFn) -> bool {
    if func.sig.inputs.len() != 1 {
        return false;
    }
    if let Some(FnArg::Typed(pat_type)) = func.sig.inputs.first() {
        is_value_slice_type(&pat_type.ty)
    } else {
        false
    }
}

fn is_value_slice_type(ty: &Type) -> bool {
    // Match &[Value] or &[mininter::value::Value]
    if let Type::Reference(r) = ty {
        if let Type::Slice(s) = &*r.elem {
            return type_ends_with(&s.elem, "Value");
        }
    }
    false
}

fn type_ends_with(ty: &Type, name: &str) -> bool {
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident == name;
        }
    }
    false
}
