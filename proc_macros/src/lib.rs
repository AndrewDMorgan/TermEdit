extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

use syn::{parse_macro_input, Expr, Token};
use syn::punctuated::Punctuated;
use syn;

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

        #hashmap.lock().unwrap().insert(
            #language,
                lua.globals().get("GetTokens").unwrap()
        );
    })
}

