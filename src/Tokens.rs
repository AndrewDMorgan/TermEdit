

// language file types for syntax highlighting
pub enum Languages {
    Rust,
    Cpp
}

const LANGS: [(Languages, &str); 3] = [
    (Languages::Cpp , "cpp"),
    (Languages::Cpp , "hpp"),
    (Languages::Rust, "rs" )
];


// token / syntax highlighting stuff idk
#[derive(Debug)]
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
    Lifetime,  // goo luck figuring this out vs. a regular string.........
    String,
    Comment,
    Null,
    Primative,
    Keyword,
}

#[derive(Clone)]
pub enum TokenFlags {
    Comment,  // has priority of everything including strings which overrule everything else
    String,  // has priority over everything (including chars)
    Char,  // has 2nd priority (over rules generics)
    Generic,
    Null,
}

pub fn GenerateTokens (text: String, fileType: &str) -> Vec <(TokenType, String)> {
    // move this to a json file so it can be customized by the user if they so chose to
    let lineBreaks = [
        " ".to_string(),
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
    ];

    let mut current = "".to_string();
    let mut tokenStrs: Vec <String> = vec!();
    for character in text.as_str().chars() {
        if !current.is_empty() && lineBreaks.contains(&character.to_string()) {
            tokenStrs.push(current.clone());
            current.clear();
            current.push(character);
            tokenStrs.push(current.clone());
            current.clear();
        } else {
            current.push(character);
            let mut valid = false;
            for breaker in &lineBreaks {
                if current.contains(breaker) {
                    valid = true;
                    break;
                }
            }
            if valid {
                tokenStrs.push(current.clone());
                current.clear();
            }
        }
    }
    tokenStrs.push(current);

    // getting any necessary flags
    let mut flags: Vec <TokenFlags> = vec!();
    let mut currentFlag = TokenFlags::Null;  // the current flag being tracked
    for (index, token) in tokenStrs.iter().enumerate() {
        let emptyString = &"".to_string();
        let nextToken = tokenStrs.get(index + 1).unwrap_or(emptyString).as_str();
        let prevToken = tokenStrs.get(index.saturating_sub(1)).unwrap_or(emptyString);
        
        let newFlag =
            match token.as_str() {
                "/" if nextToken == "/" || nextToken == "*" => TokenFlags::Comment,
                "*" if nextToken == "/" => TokenFlags::Null,
                "\"" if !matches!(currentFlag, TokenFlags::Comment | TokenFlags::Char) && prevToken != "\\" => {
                    if matches!(currentFlag, TokenFlags::String) {  TokenFlags::Null}
                    else {  TokenFlags::String}
                },
                "<" if !matches!(currentFlag, TokenFlags::Comment | TokenFlags::String | TokenFlags::Char) => TokenFlags::Generic,
                //">" if matches!(currentFlag, TokenFlags::Generic) => TokenFlags::Null,  (unfortunately the lifetimes are needed... not sure how to fix it properly without tracking more info)
                "'" if !matches!(currentFlag, TokenFlags::Comment | TokenFlags::String | TokenFlags::Generic) => {
                    if matches!(currentFlag, TokenFlags::Char) {  TokenFlags::Null}
                    else {  TokenFlags::Char  }
                },
                _ => currentFlag,
        };

        currentFlag = newFlag;
        flags.push(currentFlag.clone());
    }
    
    // track strings and track chars and generic inputs (this might allow for detecting lifetimes properly)
    let mut tokens: Vec <(TokenType, String)> = vec!();
    for (index, strToken) in tokenStrs.iter().enumerate() {
        let emptyString = &"".to_string();
        let nextToken = tokenStrs.get(index + 1).unwrap_or(emptyString).as_str();
        let prevToken = {
            if index > 0 {
                &tokenStrs[index - 1]
            } else {  &"".to_string()  }
        };
        
        let mut language = Languages::Rust;  // the default
        for (lang, extension) in LANGS {
            if extension == fileType {
                language = lang;
                break;
            }
        }

        tokens.push((
            match language {
                Languages::Cpp => match strToken.as_str() {
                    _s if matches!(flags[index], TokenFlags::Comment) => TokenType::Comment,
                    _s if matches!(flags[index], TokenFlags::String | TokenFlags::Char) => TokenType::String,
                    "if" | "for" | "while" | "in" | "else" |
                        "break" | "loop" | "goto" | "return" | "std" |
                        "const" | "static" | "template" | "continue" |
                        "include" | "#" | "alloc" | "malloc" |
                        "using" | "namespace" => TokenType::Keyword,
                    " " => TokenType::Null,
                    "int" | "float" | "double" | "string" | "char" | "short" |
                        "long" | "bool" | "unsigned" => TokenType::Primative,
                    "[" | "]" => TokenType::Bracket,
                    "(" | ")" => TokenType::Parentheses,
                    ":" => TokenType::Member,
                    s if s.chars().next().map_or(false, |c| {
                        c.is_ascii_digit()
                    }) => TokenType::Number,
                    "=" | "-" if nextToken == ">" => TokenType::Keyword,
                    ">" if prevToken == "=" => TokenType::Keyword,
                    ">" if prevToken == "-" => TokenType::Keyword,
                    "=" if prevToken == ">" || prevToken == "<" || prevToken == "=" => TokenType::Logic,
                    s if (prevToken == "&" && s == "&") || (prevToken == "|" && s == "|") => TokenType::Logic,
                    s if (nextToken == "&" && s == "&") || (nextToken == "|" && s == "|") => TokenType::Logic,
                    ">" | "<" | "false" | "true" | "!" => TokenType::Logic,
                    "=" if nextToken == "=" => TokenType::Logic,
                    "=" if prevToken == "+" || prevToken == "-" || prevToken == "*" || prevToken == "/" => TokenType::Math,
                    "=" if nextToken == "+" || nextToken == "-" || nextToken == "*" || nextToken == "/" => TokenType::Math,
                    "+" | "-" | "*" | "/" => TokenType::Math,
                    "{" | "}" | "|" => TokenType::SquirlyBracket,
                    "=" => TokenType::Assignment,
                    ";" => TokenType::Endl,
                    "&" => TokenType::Barrow,
                    "\"" | "'" => TokenType::String,
                    "public" | "private" | "this" | "class" | "struct" => TokenType::Object,
                    _s if strToken.to_uppercase() == *strToken => TokenType::Const,
                    _s if prevToken == "." && prevToken[..1].to_uppercase() == prevToken[..1] => TokenType::Method,
                    _s if prevToken == "." => TokenType::Member,
                    _s if strToken[..1].to_uppercase() == strToken[..1] => TokenType::Function,
                    _ => TokenType::Null,
                }
                _ => match strToken.as_str() {  // rust
                    _s if matches!(flags[index], TokenFlags::Comment) => TokenType::Comment,
                    _s if matches!(flags[index], TokenFlags::String | TokenFlags::Char) => TokenType::String,
                    "if" | "for" | "while" | "in" | "else" |
                        "break" | "loop" | "match" | "return" | "std" |
                        "const" | "static" | "dyn" | "type" | "continue" |
                        "use" | "mod" | "None" | "Some" | "Ok" | "Err" |
                        "async" | "await" | "default" | "derive" | "new" |
                        "as" | "?" => TokenType::Keyword,
                    " " => TokenType::Null,
                    "i32" | "isize" | "i16" | "i8" | "i128" | "i64" |
                        "u32" | "usize" | "u16" | "u8" | "u128" | "u64" | 
                        "f16" | "f32" | "f64" | "f128" | "String" |
                        "str" | "Vec" | "bool" | "char" | "Result" |
                        "Option" => TokenType::Primative,
                    "[" | "]" => TokenType::Bracket,
                    "(" | ")" => TokenType::Parentheses,
                    "#" => TokenType::Macro,
                    _s if nextToken == "!" => TokenType::Macro,
                    ":" => TokenType::Member,
                    //"" => TokenType::Variable,
                    s if s.chars().next().map_or(false, |c| {
                        c.is_ascii_digit()
                    }) => TokenType::Number,
                    "=" | "-" if nextToken == ">" => TokenType::Keyword,
                    ">" if prevToken == "=" => TokenType::Keyword,
                    ">" if prevToken == "-" => TokenType::Keyword,
                    "=" if prevToken == ">" || prevToken == "<" || prevToken == "=" => TokenType::Logic,
                    s if (prevToken == "&" && s == "&") || (prevToken == "|" && s == "|") => TokenType::Logic,
                    s if (nextToken == "&" && s == "&") || (nextToken == "|" && s == "|") => TokenType::Logic,
                    ">" | "<" | "false" | "true" | "!" => TokenType::Logic,
                    "=" if nextToken == "=" => TokenType::Logic,
                    "=" if prevToken == "+" || prevToken == "-" || prevToken == "*" || prevToken == "/" => TokenType::Math,
                    "=" if nextToken == "+" || nextToken == "-" || nextToken == "*" || nextToken == "/" => TokenType::Math,
                    "+" | "-" | "*" | "/" => TokenType::Math,
                    "{" | "}" | "|" => TokenType::SquirlyBracket,
                    "let" | "=" | "mut" => TokenType::Assignment,
                    ";" => TokenType::Endl,
                    "&" => TokenType::Barrow,
                    "'" if matches!(flags[index], TokenFlags::Generic) => TokenType::Lifetime,
                    _s if matches!(flags[index], TokenFlags::Generic) && prevToken == "'" => TokenType::Lifetime,
                    "a" | "b" if prevToken == "'" && (nextToken == "," || nextToken == ">" || nextToken == " ") => TokenType::Lifetime,
                    "\"" | "'" => TokenType::String,
                    "enum" | "pub" | "struct" | "impl" | "self" | "Self" => TokenType::Object,
                    _s if strToken.to_uppercase() == *strToken => TokenType::Const,
                    _s if prevToken == "." && prevToken[..1].to_uppercase() == prevToken[..1] => TokenType::Method,
                    _s if prevToken == "." => TokenType::Member,
                    "fn" => TokenType::Function,
                    _s if strToken[..1].to_uppercase() == strToken[..1] => TokenType::Function,
                    _ => TokenType::Null,
                },
            },
        strToken.clone()));
    }

    tokens
}



// application stuff
#[derive(Debug)]
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

        self.children[index.unwrap()].GetNode(scope)
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

const VALID_NAMES_NEXT: [&str; 4] = [
    "fn",
    "struct",
    "enum",
    "impl",
];

const VALID_NAMES_TAKE: [&str; 6] = [
    "for",
    "while",
    "if",
    "else",
    "match",
    "loop",
];

pub fn GenerateScopes (tokenLines: &[Vec <(TokenType, String)>]) -> (ScopeNode, Vec <Vec <usize>>, Vec <Vec <usize>>) {
    // tracking the scope (functions = new scope; struct/enums = new scope; for/while = new scope)
    let mut rootNode = ScopeNode {
        children: vec![],
        name: "Root".to_string(),
        start: 0,
        end: tokenLines.len().saturating_sub(1),
    };

    let mut jumps: Vec <Vec <usize>> = vec!();
    let mut linearized: Vec <Vec <usize>> = vec!();

    let mut currentScope: Vec <usize> = vec!();
    for (lineNumber, tokens) in tokenLines.iter().enumerate() {
        let mut bracketDepth = 0isize;
        // track the depth; if odd than based on the type of bracket add scope; if even do nothing (scope opened and closed on the same line)
        // use the same on functions to determin if the scope needs to continue or end on that line
        for (index, (token, name)) in tokens.iter().enumerate() {
            let (_lastToken, lastName) = {
                if index == 0 {  &(TokenType::Null, "".to_string())  }
                else {  &tokens[index - 1]  }
            };
            if !(matches!(token, TokenType::Comment) || (lastName == "\"" || lastName == "'") && matches!(token, TokenType::String)) {
                // checking bracket depth
                if name == "{" {
                    bracketDepth += 1;
                } else if name == "}" {
                    bracketDepth -= 1;
                }
            }

            // checking for something to define the name
            if matches!(token, TokenType::Keyword | TokenType::Object | TokenType::Function) {
                if !(VALID_NAMES_NEXT.contains(&name.trim()) || VALID_NAMES_TAKE.contains(&name.trim())) {
                    continue;
                }
                
                // checking the scope to see if ti continues or ends on the same line
                let mut brackDepth = 0isize;
                for (indx, (token, name)) in tokens.iter().enumerate() {
                    let (_lastToken, lastName) = {
                        if indx == 0 {  &(TokenType::Null, "".to_string())  }
                        else {  &tokens[indx - 1]  }
                    };
                    if !(matches!(token, TokenType::Comment) || lastName == "\"" && matches!(token, TokenType::String)) &&
                        indx > index 
                    {
                        if name == "{" {
                            brackDepth += 1;
                        } else if name == "}" {
                            brackDepth -= 1;
                        }
                    }
                }

                // check bracketdepth here so any overlapping marks are accounted for
                if bracketDepth < 0 && brackDepth > 0 {  // not pushing any jumps bc/ it'll be done later
                    let mut scopeCopy = currentScope.clone();
                    scopeCopy.reverse();
                    rootNode.SetEnd(&mut scopeCopy, lineNumber);
                    currentScope.pop();
                }
                
                // adding the new scope if necessary
                if brackDepth > 0 {
                    if VALID_NAMES_NEXT.contains(&name.trim()) {
                        let nextName = tokens.get(index + 2).unwrap_or(&(TokenType::Null, "".to_string())).1.clone();
                        
                        let mut scopeCopy = currentScope.clone();
                        scopeCopy.reverse();
                        let mut goodName = name.clone();
                        goodName.push(' ');
                        goodName.push_str(nextName.as_str());
                        let newScope = rootNode.Push(&mut scopeCopy, goodName, lineNumber);
                        currentScope.push(newScope);
                        linearized.push(currentScope.clone());
                        
                        bracketDepth = 0;
                        break;
                    } else if VALID_NAMES_TAKE.contains(&name.trim()) {
                        let mut scopeCopy = currentScope.clone();
                        scopeCopy.reverse();
                        let goodName = name.clone();
                        let newScope = rootNode.Push(&mut scopeCopy, goodName, lineNumber);
                        currentScope.push(newScope);
                        linearized.push(currentScope.clone());

                        bracketDepth = 0;
                        break;
                    }
                }
            }
        }
        
        // updating the scope based on the brackets
        if bracketDepth > 0 {
            let mut scopeCopy = currentScope.clone();
            scopeCopy.reverse();
            let newScope = rootNode.Push(&mut scopeCopy, "{ ... }".to_string(), lineNumber);
            currentScope.push(newScope);
            linearized.push(currentScope.clone());
            jumps.push(currentScope.clone());
        } else if bracketDepth < 0 {
            jumps.push(currentScope.clone());
            let mut scopeCopy = currentScope.clone();
            scopeCopy.reverse();
            rootNode.SetEnd(&mut scopeCopy, lineNumber);
            currentScope.pop();
        } else {
            jumps.push(currentScope.clone());
        }
    }

    (rootNode, jumps, linearized)
}
