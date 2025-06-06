use proc_macros::load_language_types;
use mlua::{Error, FromLua, Lua, Value};
use parking_lot::RwLock;
use std::sync::Arc;

// loads all the languages from the provided file
load_language_types!("data/syntaxHighlighting.json");


/// Use the following to create a valid interface. Additionally, link the script which implements this
/// trait in syntaxHighlighting.json. There has to be a wrapper method labeled GenerateScopes that
/// contains this trait. The struct that contains the trait should have the same name as the name of the language.
/// An example being the Rust enum variant which would require the struct to be called Rust.
/// ```
/// impl<F> LanguageLinterInterface for F
/// {
///     fn GenerateScopes(...) -> ... {
///         todo!()
///     }
/// }
/// ```
pub trait LanguageLinterInterface {
    fn GenerateScopes(tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
                      lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
                      outlineOriginal: &Arc <RwLock <Vec <OutlineKeyword>>>
    ) -> (ScopeNode, Vec <Vec <usize>>, Vec <Vec <usize>>);
}

pub type TraitSignature = fn (
    &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
    &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    &Arc <RwLock <Vec <OutlineKeyword>>>
) -> (ScopeNode, Vec <Vec <usize>>, Vec <Vec <usize>>);


#[derive(Debug, Clone, Default)]
pub struct LuaTuple {
    pub token: TokenType,
    pub text: String,
}

impl FromLua for LuaTuple {
    fn from_lua(values: Value, _lua: &Lua) -> mlua::Result<Self> {
        let table: mlua::Table = match values.as_table() {
            Some(t) => t.clone(),
            _ => {
                return Err(Error::FromLuaConversionError {
                    from: "MultiValue",
                    to: "Token".to_string(),
                    message: Some("Expected a Lua table".to_string()),
                })
            }
        };

        let token;
        let tokenValue: Result <Value, _> = table.get(1);
        if tokenValue.is_ok() {
            let tokenValue = tokenValue?;
            if tokenValue.is_string() {
                token = match tokenValue.as_string_lossy().unwrap_or(String::new()).as_str() {
                    "Bracket" => Ok(TokenType::Bracket),
                    "SquirlyBracket" => Ok(TokenType::SquirlyBracket),
                    "Parentheses" => Ok(TokenType::Parentheses),
                    "Variable" => Ok(TokenType::Variable),
                    "Member" => Ok(TokenType::Member),
                    "Object" => Ok(TokenType::Object),
                    "Function" => Ok(TokenType::Function),
                    "Method" => Ok(TokenType::Method),
                    "Number" => Ok(TokenType::Number),
                    "Logic" => Ok(TokenType::Logic),
                    "Math" => Ok(TokenType::Math),
                    "Assignment" => Ok(TokenType::Assignment),
                    "Endl" => Ok(TokenType::Endl),
                    "Macro" => Ok(TokenType::Macro),
                    "Const" => Ok(TokenType::Const),
                    "Barrow" => Ok(TokenType::Barrow),
                    "Lifetime" => Ok(TokenType::Lifetime),
                    "String" => Ok(TokenType::String),
                    "Comment" => Ok(TokenType::Comment),
                    "Primitive" => Ok(TokenType::Primitive),
                    "Keyword" => Ok(TokenType::Keyword),
                    "CommentLong" => Ok(TokenType::CommentLong),
                    "Unsafe" => Ok(TokenType::Unsafe),
                    "Grayed" => Ok(TokenType::Grayed),
                    _ => Ok(TokenType::Null),
                };
            } else {
                //let text = format!("Invalid token arg {:?}", tokenValue);
                //panic!("{}", text);
                //token = Err(Error::UserDataTypeMismatch);
                token = Ok(TokenType::Null)
            }
        } else {
            token = Err(Error::UserDataTypeMismatch);
        }

        let text;
        let textValue: Result <Value, _> = table.get(2);
        if textValue.is_ok() {
            let textValue = textValue?;
            if textValue.is_string() {
                text = Ok(textValue.as_string_lossy().unwrap_or(String::new()));
            } else {
                text = Err(Error::UserDataTypeMismatch);
            }
        } else {
            text = Err(Error::UserDataTypeMismatch);
        }

        Ok( LuaTuple {token: token?, text: text?} )
    }
}


// token / syntax highlighting stuff idk
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub enum TokenType {
    Bracket,
    SquirlyBracket,
    Parentheses,
    Variable,
    Member,
    Object,
    Function,
    Method,
    Number,
    Logic,
    Math,
    Assignment,
    Endl,
    Macro,
    Const,
    Barrow,
    Lifetime,
    String,
    Comment,
    #[default] Null,
    Primitive,
    Keyword,
    CommentLong,
    Unsafe,
    Grayed,
}


// tracking both the next line flags, but also individual token flags for variable/outline generation (auto complete stuff ig)
#[derive(Debug, Clone, PartialEq)]
pub enum LineTokenFlags {
    Comment,
    Parameter,
    Generic,
    List,
    String,
    // Expression,  // leaving this out for now (not sure if brackets matter here)
}

#[derive(Debug, Clone)]
pub enum OutlineType {
    Variable,
    Struct,
    Enum,
    Variant,  // the variant of enum
    Function,
    Member,
    Generic,
    Lifetime,
    Mod,
}

#[derive(Debug, Clone)]
pub struct OutlineKeyword {
    pub keyword: String,
    pub kwType: OutlineType,
    pub typedType: Option <String>,  // only for explicitly annotated types -- basic error tracking?
    pub resultType: Option <String>,  // for function outputs
    pub childKeywords: Vec <OutlineKeyword>,
    pub scope: Vec <usize>,  // for tracking private/public methods and members
    pub public: Option <bool>,  // true == public; false == private (or None)
    pub mutable: bool,  // false == no; true == yes

    // name, type; the type has to be explicitly annotated for this to be picked up
    pub parameters: Option <Vec <(String, Option <String>)>>,
    pub lineNumber: usize,
    pub implLines: Vec <usize>,
}

impl OutlineKeyword {
    pub fn EditScopes (outline: &mut [OutlineKeyword], scope: &[usize], lineNumber: usize) {
        //let mut outlineWrite = outline.write();
        for keyword in outline.iter_mut() {
            if keyword.lineNumber == lineNumber {
                keyword.scope = scope.to_owned();
                if matches!(keyword.kwType, OutlineType::Function | OutlineType::Enum | OutlineType::Struct) {
                    keyword.scope.pop();  // seems to fix things? idk
                }
                // outlineWrite is dropped
                //drop(outlineWrite);
                return;
            }
        }
        // outlineWrite is dropped
        //drop(outlineWrite);
    }

    pub fn GetValidScoped (outline: &Arc <RwLock <Vec <OutlineKeyword>>>, scope: &Vec <usize>) -> Vec <OutlineKeyword> {
        let mut valid: Vec <OutlineKeyword> = vec!();
        let outlineRead = outline.read();
        for keyword in outlineRead.iter() {
            if keyword.scope.is_empty() || scope.as_slice().starts_with(keyword.scope.as_slice()) {
                valid.push(keyword.clone());
            }
        }
        // outlineRead is dropped
        drop(outlineRead);
        valid
    }

    pub fn TryFindKeyword (outline: &Arc <RwLock <Vec <OutlineKeyword>>>, queryWord: String) -> Option <OutlineKeyword> {
        let outlineRead = outline.read();
        for keyword in outlineRead.iter() {
            if queryWord == keyword.keyword {
                return Some(keyword.clone());
            }
        }
        // outlineRead is dropped
        drop(outlineRead);
        None
    }
    pub fn TryFindKeywords (outline: &Arc <RwLock <Vec <OutlineKeyword>>>, queryWord: String) -> Vec <OutlineKeyword> {
        let mut validKeywords = vec!();
        let outlineRead = outline.read();
        for keyword in outlineRead.iter() {
            if queryWord == keyword.keyword {
                validKeywords.push(keyword.clone());
            }
        }
        // outlineRead is dropped
        drop(outlineRead);
        validKeywords
    }
}


// application stuff
#[derive(Debug, Clone, Default)]
pub struct ScopeNode {
    pub children: Vec <ScopeNode>,
    pub name: String,
    pub start: usize,
    pub end: usize,
}

impl ScopeNode {
    pub fn GetNode (&self, scope: &mut Vec <usize>) -> &ScopeNode {
        let index = scope.pop();

        if index.is_none() {
            return self;
        }

        //  !!! *error* this crashed, figure it out at some point...... (ya........)
        let node = self.children.get(index.unwrap_or(usize::MAX));
        if node.is_none() {  return self;  }  // would this work? at least to fix any crashes?
        node.unwrap().GetNode(scope)
    }

    pub fn Push (&mut self, scope: &mut Vec <usize>, name: String, start: usize) -> usize {
        let index = scope.pop();
        if index.is_none() {
            self.children.push(
                ScopeNode {
                    children: vec![],
                    name,
                    start,
                    end: 0
                }
            );
            return self.children.len() - 1;
        }

        self.children[index.unwrap()].Push(scope, name, start)
    }

    pub fn SetEnd (&mut self, scope: &mut Vec <usize>, end: usize) {
        let index = scope.pop();
        if index.is_none() {
            self.end = end;
            return;
        }

        self.children[index.unwrap()].SetEnd(scope, end);
    }
}

