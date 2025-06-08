// for the old syntax highlighting functions
//#![allow(dead_code)]

// for some reason when I set something just to pass it
// in as a parameter, it thinks it's never read even though
// it's read in the function it's passed to
#![allow(unused_assignments)]

use crate::LuaScripts;

use parking_lot::{Mutex, RwLock};
use crossbeam::thread;
use std::sync::Arc;

use crate::TokenInfo::*;


lazy_static::lazy_static! {
    static ref LINE_BREAKS: [String; 27] = [
        " ".to_string(),
        "@".to_string(),
        "?".to_string(),
        "=".to_string(),
        "|".to_string(),
        "#".to_string(),
        ".".to_string(),
        ",".to_string(),
        "(".to_string(),
        ")".to_string(),
        "[".to_string(),
        "]".to_string(),
        "{".to_string(),
        "}".to_string(),
        ";".to_string(),
        ":".to_string(),
        "!".to_string(),
        "/".to_string(),
        "+".to_string(),
        "-".to_string(),
        "*".to_string(),
        "&".to_string(),
        "'".to_string(),
        "\"".to_string(),
        "<".to_string(),
        ">".to_string(),
        "$".to_string(),
    ];
}

fn GenerateTokenStrs (text: &str) -> Arc <Mutex <Vec <String>>> {
    let mut current = "".to_string();
    let tokenStrs: Arc <Mutex <Vec <String>>> = Arc::new(Mutex::new(vec![]));
    let tokenStrsClone = Arc::clone(&tokenStrs);
    let mut tokenStrsClone = tokenStrsClone.lock();
    for character in text.chars() {
        if !current.is_empty() && LINE_BREAKS.contains(&character.to_string()) {
            tokenStrsClone.push(current.clone());
            current.clear();
            current.push(character);
            tokenStrsClone.push(current.clone());
            current.clear();
            continue
        }

        current.push(character);
        let mut valid = false;
        for breaker in LINE_BREAKS.iter() {
            if !current.contains(breaker) {  continue;  }
            valid = true;
            break;
        }

        if !valid {  continue;  }
        tokenStrsClone.push(current.clone());
        current.clear();
    }
    tokenStrsClone.push(current);
    tokenStrs
}

fn GenerateLineTokenFlags (
    lineTokenFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    tokens: &[LuaTuple],
    previousFlagSet: &[LineTokenFlags],
    text: &str,
    lineNumber: usize,
) {
    // generating the new set
    let mut currentFlags = previousFlagSet.to_owned();
    for (index, token) in tokens.iter().enumerate() {
        let lastTokenText = tokens[index.saturating_sub(1)].text.clone();

        // not a great system for generics, but hopefully it'll work for now
        match token.text.as_str() {
            // removing comments can still happen within // comments
            "/" if lastTokenText == "*" && currentFlags.contains(&LineTokenFlags::Comment) => {
                RemoveLineFlag(&mut currentFlags, LineTokenFlags::Comment);
            }

            _ if matches!(token.token, TokenType::Comment) => {},
            // generics
            "<" if text.contains("fn") ||
                text.contains("struct") ||
                text.contains("impl") => { currentFlags.push(LineTokenFlags::Generic); }
            ">" if currentFlags.contains(&LineTokenFlags::Generic) => {
                RemoveLineFlag(&mut currentFlags, LineTokenFlags::Generic);
            }

            // strings (and char-strings ig)
            "\"" | "'" => {
                if currentFlags.contains(&LineTokenFlags::String) {
                    RemoveLineFlag(&mut currentFlags, LineTokenFlags::String);
                } else { currentFlags.push(LineTokenFlags::String); }
            }

            // Parameters
            "(" => { currentFlags.push(LineTokenFlags::Parameter); }
            ")" if currentFlags.contains(&LineTokenFlags::Parameter) => {
                RemoveLineFlag(&mut currentFlags, LineTokenFlags::Parameter);
            }

            // List
            "[" => { currentFlags.push(LineTokenFlags::List); }
            "]" if currentFlags.contains(&LineTokenFlags::List) => {
                RemoveLineFlag(&mut currentFlags, LineTokenFlags::List);
            }

            // Comments (only /* && */  not //)
            "*" if lastTokenText == "/" => { currentFlags.push(LineTokenFlags::Comment); }

            _ => {}
        }

        // the .write is being called in a loop, but writes block all other threads and
        // this is async so it's better to stall this one than the main one
        let mut tokenFlagsWrite = lineTokenFlags.write();
        tokenFlagsWrite[lineNumber].push(currentFlags.clone());
        drop(tokenFlagsWrite);  // dropped the write
    }
}

pub async fn GenerateTokens (
    text: String, fileType: &str,
    lineTokenFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    lineNumber: usize,
    _outline: &Arc <RwLock <Vec <OutlineKeyword>>>,
    luaSyntaxHighlightScripts: &LuaScripts,
) -> Vec <LuaTuple> {
    let tokenStrs = GenerateTokenStrs(text.as_str());

    // start a lua execution thread here
    let mut language = Languages::Null;  // the default
    for (lang, extension) in LANGS.iter() {
        if *extension == fileType {
            language = *lang;
            break;
        }
    }

    // calling the lua script (dealing with annoying async stuff)
    let highlightingScript = luaSyntaxHighlightScripts.lock();
    let script: Arc <&mlua::Function> =
        if highlightingScript.contains_key(&language) {
            Arc::clone(&Arc::from(&highlightingScript[&language]))
        } else {
            Arc::clone(&Arc::from(&highlightingScript[&Languages::Null]))
    };

    // spawning the threads
    let mut tokens: Vec <LuaTuple> = vec!();
    let mut line: Vec <Vec <LineTokenFlags>> = vec!();  // all of these have to be pre-allocated (and initialized)
    let mut previousFlagSet: &Vec <LineTokenFlags> = &vec!();
    // ignoring errors (hopefully it'll just make everything a null token?)
    let _ = thread::scope(|s| {
        let tokensWrapped: Arc<Mutex<Vec <LuaTuple>>> = Arc::new(Mutex::new(vec!()));

        let scriptClone: Arc<&mlua::Function> = Arc::clone(&script);
        let tokensClone = Arc::clone(&tokensWrapped);
        let tokenStrsClone = Arc::clone(&tokenStrs);
        let handle = s.spawn(move |_| {
            let numStrs = {  tokenStrsClone.lock().len()  };
            let input = tokenStrsClone.lock().clone();
            let result = scriptClone.call(input);
            *tokensClone.lock() = result.unwrap_or(vec![LuaTuple::default(); numStrs]);
        });


        // intermediary code
        let mut tokenFlagsWrite = lineTokenFlags.write();
        while tokenFlagsWrite.len() < lineNumber + 1 {
            tokenFlagsWrite.push(vec!());  // new line
        }
        drop(tokenFlagsWrite);  // dropped the .write

        // dealing with the line token stuff (running while lua is doing async stuff)
        let tokenFlagsRead = lineTokenFlags.read();
        if lineNumber != 0 && !tokenFlagsRead[lineNumber - 1].is_empty() {
            previousFlagSet = {
                line = tokenFlagsRead[lineNumber - 1].clone();
                line.last().unwrap()
            };
        }
        drop(tokenFlagsRead);  // dropped the read

        let mut tokenFlagsWrite = lineTokenFlags.write();
        tokenFlagsWrite[lineNumber].clear();
        drop(tokenFlagsWrite);  // dropped the write


        // joining the thread
        let _ = handle.join();
        tokens = Arc::try_unwrap(tokensWrapped)
            .unwrap()
            .into_inner();
    });

    // handling everything that requires the tokens to be calculated
    GenerateLineTokenFlags(lineTokenFlags,
                           &tokens,
                           previousFlagSet,
                           &text,
                           lineNumber
    );

    let tokenFlagsRead = lineTokenFlags.read();
    for (i, token) in tokens.iter_mut().enumerate() {
        if tokenFlagsRead[lineNumber][i].contains(&LineTokenFlags::Comment) {
            token.token = TokenType::CommentLong;
        }
    }
    drop(tokenFlagsRead);  // dropped the read

    tokens
}

fn RemoveLineFlag(currentFlags: &mut Vec<LineTokenFlags>, removeFlag: LineTokenFlags) {
    for (i, flag) in currentFlags.iter().enumerate() {
        if *flag == removeFlag {
            currentFlags.remove(i);
            break;
        }
    }
}


// using a proc mac to load in all traits
pub mod Linters {
    // including the file paths/linking them
    use proc_macros::link_linters;
    use crate::TokenInfo::*;
    use parking_lot::RwLock;
    use std::sync::Arc;

    link_linters!("data/syntaxHighlighting.json");

    // a final wrapper on the trait for easier access
    pub fn GenerateScopes(tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
                          lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
                          outlineOriginal: &Arc <RwLock <Vec <OutlineKeyword>>>,
                          fileType: &str
    ) -> (ScopeNode, Vec <Vec <usize>>, Vec <Vec <usize>>) {
        let mut language = Languages::Null;  // the default
        for (lang, extension) in LANGS.iter() {
            if *extension == fileType {
                language = *lang;
                break;
            }
        }
        if language != Languages::Null {
            let function = LANG_LINTERS.iter().find_map(|v| {
                if v.0 == language {  Some(v.1)  }
                else {  None  }
            });
            if let Some(function) = function {
                return function(tokenLines, lineFlags, outlineOriginal);
            }
        }
        // an empty array (which would stand for an empty file/no usable linted data
        (ScopeNode::default(), vec![], vec![])
    }
}
