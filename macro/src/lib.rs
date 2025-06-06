use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{FnArg, ItemFn, LitStr, PathArguments, ReturnType, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn java_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    let return_ty = match get_result_type(&input_fn.sig.output) {
        None => input_fn.sig.output.to_token_stream(),
        Some(ty) => ty.to_token_stream(),
    };
    let class_path = parse_macro_input!(attr as LitStr).value();

    let fn_name = &input_fn.sig.ident;
    let new_fn_name_str = format!("Java_{}_{}", class_path.replace('.', "_"), fn_name);
    let new_fn_name = format_ident!("{}", new_fn_name_str);

    let mut new_inputs = Punctuated::<FnArg, Comma>::new();
    new_inputs.push(syn::parse_quote! { mut env: jni::JNIEnv });
    new_inputs.push(syn::parse_quote! { this: jni::objects::JObject });

    for arg in &input_fn.sig.inputs {
        new_inputs.push(arg.clone().into());
    }

    input_fn
        .sig
        .inputs
        .insert(0, syn::parse_quote! { env: &mut jni::JNIEnv });
    input_fn
        .sig
        .inputs
        .insert(1, syn::parse_quote! { this: jni::objects::JObject });

    let arg_idents: Vec<_> = new_inputs
        .iter()
        .skip(2)
        .map(|arg| match arg {
            FnArg::Typed(pat_type) => match &*pat_type.pat {
                syn::Pat::Ident(ident) => ident.ident.clone(),
                _ => panic!("Expected ident"),
            },
            _ => panic!("Unexpected receiver"),
        })
        .collect();

    let call_args = quote! {
        &mut env, this, #(#arg_idents),*
    };

    let wrapped = quote! {
        let warp = std::panic::AssertUnwindSafe(|| { #fn_name(#call_args) });
        match std::panic::catch_unwind(warp) {
            Ok(result) => { result }
            Err(err) => {
                if env.exception_check().unwrap_or_default() {
                    _ = env.exception_describe();
                }
                let msg = if let Some(s) = err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic payload type".to_string()
                };
                _ = env.throw_new("Ljava/lang/Exception;", msg);
                <#return_ty as Default>::default()
            }
        }
    };

    let output = quote! {
        #[jni_macro::handle_result]
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "system" fn #new_fn_name(#new_inputs) -> #return_ty {
            #wrapped
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn handle_result(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    match get_result_type(&input_fn.sig.output) {
        None => quote!(#input_fn).into(),
        Some(ty) => {
            let ok = ty.to_token_stream();
            let stmts = &input_fn.block.stmts;

            let new_block = quote!({
                let r:  Result<#ty> = (||{ #(#stmts)* })();
                match r {
                    Ok(val) => {
                        val
                    },
                    Err(err) => {
                        if env.exception_check().unwrap_or_default() {
                            _ = env.exception_describe();
                        }
                        _ = env.throw_new("Ljava/lang/Exception;", format!("{:?}", err));
                        <#ok as Default>::default()
                    }
                }
            });
            input_fn.sig.output = syn::parse_quote!(-> #ok);
            input_fn.block = syn::parse_quote!(#new_block);
            quote!(#input_fn).into()
        }
    }
}

fn get_result_type(ty: &ReturnType) -> Option<&Type> {
    let ty = match ty {
        ReturnType::Default => return None,
        ReturnType::Type(_, ty) => ty.as_ref(),
    };

    let type_path = match ty {
        Type::Path(type_path) => type_path,
        _ => return None,
    };

    let segments = &type_path.path.segments;
    let result = segments.iter().find(|s| s.ident == "Result")?;
    let args = match &result.arguments {
        PathArguments::AngleBracketed(args) => args,
        _ => return None,
    };

    let first_args = args.args.first()?;
    if let syn::GenericArgument::Type(ty) = first_args {
        return Some(ty);
    }
    None
}
