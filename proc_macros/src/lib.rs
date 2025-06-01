//#![allow(non_snake_case)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

use syn::{parse_macro_input, Expr, Token};
use syn::punctuated::Punctuated;
use syn;

// loads a lua script for syntax highlighting into the hash-map
#[proc_macro]
pub fn load_lua_script (input: TokenStream) -> TokenStream {
    let args: Punctuated<Expr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);

    let hashmap = &args[0];
    let language = &args[1];
    let path = &args[2];

    TokenStream::from (quote! {
        let lua = mlua::Lua::new();
        lua.load(
            std::fs::read_to_string(#path)?
        ).exec().unwrap();

        #hashmap.lock().insert(
            #language,
                lua.globals().get("GetTokens").unwrap()
        );
    })
}

/// Expands to a call to the Colorize trait. Colorize is implemented by default for...
/// * Colored
/// * String
/// * str
/// # Parameters
/// - value of type T with trait Colorize
/// - n ColorType variants (n: 0 - ∞)
/// # Example
/// ```
/// proc_macros::color!("Hello World", White, Bold, Underline);
/// proc_macros::color!("Hello World");  // converts to Colored without applying modifiers
/// ```
#[proc_macro]
pub fn color (input: TokenStream) -> TokenStream {
    let args: Punctuated<Expr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);
    let string = &args[0];
    let variants: Vec <_> = args.iter().skip(1).map(|ident| quote! { ColorType::#ident }).collect();

    if variants.len() == 1 {
        let arg = &args[1];
        return TokenStream::from (quote! {
            #string.Colorize(ColorType::#arg)
        })
    }
    TokenStream::from (quote! {
        #string.Colorizes(vec![#(#variants),*])
    })
}

