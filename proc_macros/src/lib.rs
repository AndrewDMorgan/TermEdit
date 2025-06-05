use serde_json::Value;
use std::io::Read;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

use syn::{parse_macro_input, Expr, Token};
use syn::punctuated::Punctuated;

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

fn load_json (file_name: &str) -> Value {
    let mut file = std::fs::File::open(file_name).unwrap();  // Open the file
    let mut file_content = String::new();
    file.read_to_string(&mut file_content).unwrap();  // Read content into a string
    serde_json::from_str( &file_content ).unwrap()
}

/// takes the path to a json file that contains all the scripts. Using that, all the scripts are loaded
/// inline to avoid lifetime errors while still allowing a more dynamic approach
#[proc_macro]
pub fn load_lua_scripts (input: TokenStream) -> TokenStream {
    let args: Punctuated<Expr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);

    let hashmap = &args[0];
    let path = &args[1];

    // gathering the data
    let path_str = match path {
        Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) => lit_str.value(),
        _ => panic!("Expected a string literal as input, like \"file.json\""),
    };
    let content = load_json(&path_str);

    let (scripts, endings) = (
        content.get("scripts"),
        content.get("file_endings")
    );
    let (scripts, endings) = (scripts.unwrap().as_array(), endings.unwrap().as_array());
    let (scripts, endings) = (scripts.unwrap(), endings.unwrap());

    let mut language_scripts = quote! {};
    for (i, script) in scripts.iter().enumerate() {
        let script = script.as_str().unwrap().to_string();
        let ending = syn::parse_str::<Expr>(endings[i].as_array().unwrap()[1].as_str().unwrap()).unwrap();
        language_scripts = quote! {
            #language_scripts
            load_lua_script!(
                #hashmap,
                Languages::#ending,
                #script
            );
        };
    }
    //println!("{}", language_scripts.to_string());
    TokenStream::from(language_scripts)
}


/// based on a given json file, this will load all the language types into an enum
#[proc_macro]
pub fn load_language_types (input: TokenStream) -> TokenStream {
    let args: Punctuated<Expr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);
    let path = &args[0];
    let path_str = match path {
        Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) => lit_str.value(),
        _ => panic!("Expected a string literal as input, like \"file.json\""),
    };
    let content = load_json(&path_str);

    let endings = content.get("file_endings");
    let endings = endings.unwrap().as_array();
    let endings = endings.unwrap();

    let mut size = 0usize;
    let mut langs = quote! {};
    let mut lang_enum = quote! {
        Null
    };
    for lang in endings {
        let lang_info = lang.as_array().unwrap();
        let endings = lang_info[0].as_array().unwrap();
        let language = syn::parse_str::<Expr>(lang_info[1].as_str().unwrap()).unwrap();
        for extension in endings {
            let extension = extension.as_str().unwrap();
            if size == 0 {
                langs = quote! {  (Languages::#language, #extension)  };
            } else {
                langs = quote! {
                    #langs, (Languages::#language, #extension)
                };
            } size += 1;
        }
        lang_enum = quote! {
            #lang_enum, #language
        };
    }

    TokenStream::from(quote! {
        #[derive(Clone, Hash, PartialEq, Eq, Debug, Copy)]
        pub enum Languages {
            #lang_enum
        }
        pub static LANGS: [(Languages, &str); #size] = [#langs];
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

