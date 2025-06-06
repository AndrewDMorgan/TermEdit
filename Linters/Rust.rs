use crate::TokenInfo::*;
use parking_lot::RwLock;
use std::sync::Arc;


fn HandleKeyword (
    tokens: &[LuaTuple],
    lineTokenFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    currentContainer: &mut Option <OutlineKeyword>,
    text: &str,
    tokenText: &str,
    lineNumber: usize,
    nonSet: &mut bool,
    prevTokens: &[&TokenType],
    index: usize,
) {
    let mut passedEq = false;
    match tokenText {
        "=" => {  passedEq = true;  },
        "struct" | "enum" | "fn" | "mod" => {
            let keyword = OutlineKeyword {
                keyword: String::new(),
                kwType: match tokenText {
                    "mod" => OutlineType::Mod,
                    "struct" => OutlineType::Struct,
                    "fn" => OutlineType::Function,
                    _ => OutlineType::Enum,
                },
                typedType: None,
                resultType: None,
                childKeywords: vec!(),
                scope: vec!(),
                public: Some(text.contains("pub")),
                mutable: false,
                parameters: None,
                lineNumber,
                implLines: vec!(),
            };

            currentContainer.replace(keyword);
        },
        "let" | "static" | "const" if currentContainer.is_none() => {
            let keyword = OutlineKeyword {
                keyword: String::new(),
                kwType: OutlineType::Variable,
                typedType: None,
                resultType: None,
                childKeywords: vec!(),  // figure this out :(    no clue how to track the children
                scope: vec!(),
                public: None,
                mutable: matches!(tokenText, "static" | "const"),
                parameters: None,
                lineNumber,
                implLines: vec!(),
            };

            currentContainer.replace(keyword);
        },
        "mut" if currentContainer.is_some() => {
            if let Some(keyword) = currentContainer {
                keyword.mutable = true;
            }
        },
        // the conditions seem to screen fine when doing an if or while let (and other random things)
        txt if !matches!(txt, " " | "(" | "Some" | "Ok" | "mut") &&
            currentContainer.is_some()
        => HandleBinding(
            tokens,
            lineTokenFlags,
            currentContainer,
            txt,
            tokenText,
            lineNumber,
            nonSet,
            prevTokens,
            index,
            passedEq),
        _ => {}
    }
}

fn HandleBinding (
    tokens: &[LuaTuple],
    lineTokenFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    mut currentContainer: &mut Option <OutlineKeyword>,
    txt: &str,
    tokenText: &str,
    lineNumber: usize,
    nonSet: &mut bool,
    prevTokens: &[&TokenType],
    index: usize,
    passedEq: bool
) {
    {
        if txt.get(0..1).unwrap_or("") == "_"  {
            *currentContainer = None;
            *nonSet = true;
        }

        if let Some(keyword) = &mut currentContainer {
            if keyword.keyword.is_empty() {
                keyword.keyword = tokenText.to_owned();
            } else if prevTokens.len() > 1 &&
                tokens[index.saturating_sub(2)].text == ":" &&
                !passedEq && keyword.typedType.is_none()
            {
                if matches!(keyword.kwType, OutlineType::Function) {
                    HandleFunctionDef(
                        keyword,
                        tokens,
                        lineTokenFlags,
                        tokenText,
                        index,
                        lineNumber
                    );
                } else {
                    keyword.typedType = Some(tokenText.to_owned());
                }
            }
        }
    }
}

fn HandleFunctionDef (
    keyword: &mut OutlineKeyword,
    tokens: &[LuaTuple],
    lineTokenFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    tokenText: &str,
    index: usize,
    lineNumber: usize
) {
    if keyword.parameters.is_none() {  keyword.parameters = Some(vec!());  }
    if let Some(parameters) = &mut keyword.parameters {
        parameters.push((
            tokens[index.saturating_sub(3)].text.clone(),
            Some({  // collecting all terms until the parameter field ends or a ','
                let mut text = tokenText.to_owned();
                let tokenFlagsRead = lineTokenFlags.read();
                for (nextIndex, item) in tokens.iter().enumerate().skip(index+1) {
                    if item.text == "," ||
                        !tokenFlagsRead[lineNumber][nextIndex]
                            .contains(&LineTokenFlags::Parameter)
                    {  break;  }
                    text.push_str(&item.text.clone());
                }
                // tokenFlagsRead is dropped
                drop(tokenFlagsRead);
                text
            })
        ));
    }
}

fn GetImplKeywordIndex (outline: &mut [OutlineKeyword]) -> Vec <usize> {
    // make this a non-clone at some point; me lazy rn (always lazy actually...)    I actually did it!
    let mut implKeywordIndex: Vec <usize> = vec![];

    //let mut outlineWrite = outline.write();
    for (index, keyword) in outline.iter_mut().enumerate() {
        // handling impls and mods
        match keyword.kwType {
            OutlineType::Enum | OutlineType::Function |
            OutlineType::Struct | OutlineType::Mod => {
                // checking each successive scope
                if !keyword.scope.is_empty() {  // 1 for the impl and 1 for the method? idk
                    implKeywordIndex.push(index);
                }
            }
            _ => {}
        }
    }
    // outlineWrite is dropped here
    //drop(outlineWrite);
    implKeywordIndex
}

fn HandleKeywordIndexes (
    outline: &mut [OutlineKeyword],
    implKeywordIndex: Vec <usize>,
    scopeJumps: &[Vec <usize>],
    root: &ScopeNode
) {
    for keywordIndex in implKeywordIndex {
        //let outlineRead = outline.read();

        // the first scope should be the impl or mod, right?
        let keyword = outline[keywordIndex].clone();
        let mut newScope: Option <Vec <usize>> = None;

        //if keywordIndex >= outline.len() || outline[keywordIndex].scope.is_empty() {  return;  }
        //if outline[keywordIndex].scope[0] >= root.children.len() {  return;  }
        if root.children.len() < outline[keywordIndex].scope[0] {  continue;  }
        let scopeStart = root.children[outline[keywordIndex].scope[0]].start;

        //drop(outlineRead);
        //let mut outlineWrite = outline;//.write();

        for otherKeyword in outline.iter_mut() {
            if otherKeyword.implLines.contains(&scopeStart) {
                otherKeyword.childKeywords.push(keyword);
                newScope = Some(scopeJumps[otherKeyword.lineNumber].clone());
                break;
            }
        }

        if let Some(newScope) = newScope {
            outline[keywordIndex].scope = newScope;
        }

        // makings ure there aren't any active writes
        //drop(outlineWrite);
    }
}

fn HandleKeywords (
    tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
    lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    outline: &mut [OutlineKeyword],
    scopeJumps: &[Vec <usize>],
    root: &ScopeNode
) -> Vec <OutlineKeyword> {
    // getting a write channel for outline
    // this will block any read calls until completion
    // anything inbetween this and the dropping of its memory
    // should be optimized and performant to not stall
    // the main thread (which calls lots of reads)
    //let outlineRead = outline.read();
    //let outlineSize = outline.len();
    // write is used on this so reading has to be very sparing and controlled
    //drop(outlineRead);

    // !!!the memory leak has to due with this vector
    let mut newKeywords: Vec <OutlineKeyword> = Vec::new();
    for keyword in outline.iter_mut() {
        //{
        //let outlineRead = outline.read();
        //let keywordOption = &outline.get(keywordIndex);
        //if keywordOption.is_none() {
        // error; probably an out of data thread (there should be future ones to take over)
        //drop(outlineRead);
        //    return vec!();
        //}
        //let keyword = keywordOption.unwrap();

        if !matches!(keyword.kwType, OutlineType::Enum | OutlineType::Struct)
        { continue; }
        //}  // outline read is dropped; same with keyword

        //let mut outlineWrite = outline.write();
        if !keyword.childKeywords.is_empty() {
            keyword.childKeywords.clear()
        }
        // freeing the .write to allow reading
        //drop(outlineWrite);

        // getting the following members
        //let outlineRead = outline.read();
        if keyword.lineNumber > scopeJumps.len() {  continue;  }
        //drop(outlineRead);  // dropping the read

        // there are no active reads or writes here (in terms of local)
        // the memory leak is inside here
        HandleKeywordsLoop(
            tokenLines,
            lineFlags,
            scopeJumps,
            // tracking the index and outline to allow
            // more controlled .read's and .writes
            keyword,
            &mut newKeywords,
            root
        );
    } newKeywords
}

fn HandleKeywordsLoop (
    tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
    lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    scopeJumps: &[Vec <usize>],
    keyword: &mut OutlineKeyword,
    //(outline, keywordIndex): (&mut Vec <OutlineKeyword>, usize),
    newKeywords: &mut Vec<OutlineKeyword>,
    root: &ScopeNode,
) {
    //let outlineRead = outline.read();
    //drop(outlineRead);  // dropping the read
    // no local reads

    'lines: for lineNumber in
        keyword.lineNumber+1..
            ({
                let mut scope = scopeJumps[keyword.lineNumber].clone();
                scope.reverse();
                root.GetNode(&mut scope).end
            })
    {
        //let tokenLinesRead = tokenLines.read();

        let mut public = false;
        let mut currentContainer: Option <OutlineKeyword> = None;
        // this is the issue right here
        let tokensSize = tokenLines.read()[lineNumber].len();
        for index in 0..tokensSize {
            //for (index, token) in tokenLines.read()[lineNumber].iter().enumerate() {
            //let tokenText: String;
            {
                //if lineNumber > tokenLines.read().len() || index > tokenLines.read()[lineNumber].len() {  return;  }
                // index error
                let tokenLinesRead = tokenLines.read();
                //if lineNumber >= tokenLinesRead.len() || index >= tokenLinesRead[lineNumber].len() {  return;  }
                if tokenLinesRead[lineNumber][index].text == "}" { break 'lines; }
                else if tokenLinesRead[lineNumber][index].text == "pub" { public = true; }

                //tokenText = tokenLinesRead[lineNumber][index].text.clone();
                drop(tokenLinesRead);
            }
            if matches!(tokenLines.read()[lineNumber][index].token, TokenType::Comment | TokenType::String)
            {  continue;  }

            // no active read's or writes here (safe to continue)
            // wrong, the iter takes a reference.... (fixed now)
            // the memory leak is inside here
            HandleScopeChecks(
                tokenLines,
                lineFlags,
                &mut currentContainer,
                scopeJumps,
                keyword,
                index,
                index,
                lineNumber,
                public
            );
        }

        // dropping the read channel/guard (bruh, clearly there was a read.....) me dumb
        //drop(tokenLinesRead);

        if let Some(container) = currentContainer {
            // !!!!This next line is the continuation of the leak
            newKeywords.push(container.clone());

            //let mut outlineWrite = outline.write();
            keyword.childKeywords.push(container);  // adding the variant
            //drop(outlineWrite);  // dropping the .write
        }
    }
}

fn HandleScopeChecks (
    tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
    lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    currentContainer: &mut Option <OutlineKeyword>,
    scopeJumps: &[Vec <usize>],
    keyword: &mut OutlineKeyword,
    textIndex: usize,
    index: usize,
    lineNumber: usize,
    public: bool
) {
    // getting than dropping a read channel
    //let lineFlagsRead = lineFlags.read();
    let condition = lineFlags.read()[lineNumber][index].contains(&LineTokenFlags::Parameter);
    //drop(lineFlagsRead);  // quickly dropped

    if condition && currentContainer.is_some() {
        // no active local read's or writes
        // !the memory leak isn't in here. It's somewhere else in this function
        HandleKeywordParameterSet(
            tokenLines,
            lineFlags,
            currentContainer,
            keyword,
            textIndex,
            index,
            lineNumber
        );
    } else if !matches!(tokenLines.read()[lineNumber][textIndex].text.as_str(), " " | "(" | "Some" | "Ok" | "_" | "mut" | "," | "pub") && currentContainer.is_none() {
        //let outlineRead = outline.read();
        let newKey = OutlineKeyword {
            keyword: tokenLines.read()[lineNumber][textIndex].text.clone(),
            kwType: {
                if matches!(keyword.kwType, OutlineType::Enum) {
                    OutlineType::Variant
                } else {
                    OutlineType::Member
                }
            },
            typedType: None,
            resultType: None,
            childKeywords: vec!(),
            scope: scopeJumps[keyword.lineNumber].clone(),
            public: Some(public),  // it inherits the parents publicity
            mutable: false,
            parameters: None,
            lineNumber,
            implLines: vec!(),
        };
        //drop(outlineRead);  // dropped (no functions calls are between creation & death)
        // !!!!!this line is the root of the memory leak
        currentContainer.replace(newKey);
    }
}

fn HandleKeywordParameterSet (
    tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
    lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    mut currentContainer: &mut Option <OutlineKeyword>,
    keyword: &mut OutlineKeyword,
    textIndex: usize,
    index: usize,
    lineNumber: usize,
) {
    let mut parameterType = String::new();
    if tokenLines.read()[lineNumber][textIndex].text == "(" {
        // no local active reads or writes
        HandleKeywordParameter(
            tokenLines,
            lineFlags,
            currentContainer,
            keyword,
            parameterType,
            index,
            lineNumber
        );
    } else if tokenLines.read()[lineNumber][textIndex].text == ":" {
        // opening reads for lineFlags and tokenLines
        let lineFlagsRead = lineFlags.read();
        let tokenLinesRead = tokenLines.read();

        // !!! There are active read's so don't call functions that need to read or write from these

        for newCharIndex in index + 2..tokenLinesRead[lineNumber].len() {
            if lineFlagsRead[lineNumber][index].contains(&LineTokenFlags::Comment) ||
                tokenLinesRead[lineNumber][textIndex].text == "," {  break;  }
            let string = &tokenLinesRead[lineNumber][newCharIndex].text.clone();
            parameterType.push_str(string);
        }
        // dropping the read for lineFlags
        drop(lineFlagsRead);

        if let Some(container) = &mut currentContainer {
            container.parameters = Some(
                vec![(
                    tokenLinesRead[lineNumber][index.saturating_sub(1)].text.clone(),
                    Some(parameterType)
                )]
            );
        }
        // the read for tokenLines drops here
        drop(tokenLinesRead);
    }
}

fn HandleKeywordParameter (
    tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
    lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    mut currentContainer: &mut Option <OutlineKeyword>,
    keyword: &mut OutlineKeyword,
    mut parameterType: String,
    index: usize,
    lineNumber: usize,
) {
    let tokenLinesRead = tokenLines.read();
    let lineFlagsRead = lineFlags.read();

    // getting the parameters type
    for newCharIndex in index + 1..tokenLinesRead[lineNumber].len() {
        if !lineFlagsRead[lineNumber][index].contains(&LineTokenFlags::Parameter) {
            break;
        }
        parameterType.push_str(&tokenLinesRead[lineNumber][newCharIndex].text.clone());
    }
    drop(tokenLinesRead);
    drop(lineFlagsRead);

    // active write for outline here
    //let mut outlineWrite = outline.write();

    if keyword.parameters.is_none() {  keyword.parameters = Some(Vec::new());  }

    if let Some(container) = &mut currentContainer {
        let params = vec![(container.keyword.clone(), Some(parameterType))];
        if let Some(parameters) = &mut keyword.parameters {
            for param in params {
                parameters.push(param);
            }
        }
        //container.parameters = Some(params);  // I don't think this line is needed?
    }
    // making sure no writes are active
    //drop(outlineWrite);
}

pub fn UpdateKeywordOutline (
    tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
    lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    outline: &mut Vec <OutlineKeyword>,
    scopeJumps: &[Vec <usize>],
    root: &ScopeNode
) {
    let implKeywordIndex = GetImplKeywordIndex(outline);
    HandleKeywordIndexes(outline, implKeywordIndex, scopeJumps, root);

    // it's in here... (the mem leak)
    let mut newKeywords = HandleKeywords(tokenLines, lineFlags, outline, scopeJumps, root);

    //let mut outlineWrite = outline.write();
    while let Some(newKeyword) = newKeywords.pop() {
        outline.push(newKeyword);
    }
    // making sure there are no active writes
    //drop(outlineWrite);
}


fn HandleDefinitions(
    tokens: &[LuaTuple],
    lineTokenFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
    outline: &mut Vec <OutlineKeyword>,
    text: &str,
    lineNumber: usize
) {
    let mut nonSet = false;
    let mut prevTokens: Vec <&TokenType> = vec!();
    let mut currentContainer: Option <OutlineKeyword> = None;
    for (index, token) in tokens.iter().enumerate() {
        // dealing with the outline portion now... (does this need to be in another for-loop?)
        if !nonSet {
            HandleKeyword(
                tokens,
                lineTokenFlags,
                &mut currentContainer,
                text,
                &token.text,
                lineNumber,
                &mut nonSet,
                &prevTokens,
                index
            );
        }

        prevTokens.push(&token.token);
    }
    let mut newOutline: Vec <OutlineKeyword> = vec!();
    //let outlineRead = outline.read();
    for keyWord in outline.iter() {
        if keyWord.lineNumber == lineNumber {  continue;  }
        newOutline.push(keyWord.clone());
    }
    //drop(outlineRead);  // dropped the read

    //let mut outlineWrite = outline.write();
    outline.clear();  // clearing so this is fine
    for keyWord in newOutline {
        outline.push(keyWord);
    }
    //drop(outlineWrite);  // dropped the .write

    HandleImpl(currentContainer, outline, tokens, text, lineNumber);
}

fn HandleImpl (
    currentContainer: Option <OutlineKeyword>,
    outline: &mut Vec <OutlineKeyword>,
    tokens: &[LuaTuple],
    text: &str,
    lineNumber: usize
) {
    if let Some(container) = currentContainer {
        // writing to outline; drops in the same line as called
        // should be fine sense this is called right after being cleared
        outline.push(container);
    } else if text.contains("impl") {
        let mut queryName = None;
        for index in (0..tokens.len()).rev() {
            if !matches!(tokens[index].token, TokenType::Function) {  continue;  }
            queryName = Some(tokens[index].text.clone());
        }
        HandleGettingImpl(outline, &queryName, lineNumber);
    }
}

fn HandleGettingImpl (
    outline: &mut [OutlineKeyword],
    queryName: &Option <String>,
    lineNumber: usize
) {
    if let Some(queryName) = queryName {
        // was just cleared so should be fine
        let outlineWrite = outline;
        for container in outlineWrite.iter_mut() {
            if container.keyword != *queryName {  continue;  }
            container.implLines.push(lineNumber);
            break;
        }
        // outlineWrite is dropped here
        //drop(outlineWrite);
    }
}

static VALID_NAMES_NEXT: [&str; 5] = [
    "fn",
    "struct",
    "enum",
    "impl",
    "mod",
];

static VALID_NAMES_TAKE: [&str; 6] = [
    "for",
    "while",
    "if",
    "else",
    "match",
    "loop",
];

pub struct Rust {}
impl LanguageLinterInterface for Rust {
    fn GenerateScopes (
        tokenLines: &Arc <RwLock <Vec <Vec <LuaTuple>>>>,
        lineFlags: &Arc <RwLock <Vec <Vec <Vec <LineTokenFlags>>>>>,
        outlineOriginal: &Arc <RwLock <Vec <OutlineKeyword>>>,
    ) -> (ScopeNode, Vec <Vec <usize>>, Vec <Vec <usize>>) {
        // creating a clone of outline to avoid external influence that may lead to a memory leak
        // beforehand multiple threads were appending to the outline at the same time
        // duplicating various elements
        let outline = &mut outlineOriginal.read().clone();
        // confirmed, the memory leak is inside here

        // opening a reading channel for tokenLines (writing is very rare, so I'm not worried)
        // this pauses until any write channels are closed (there can be multiple read channels)
        //let lineFlagsRead = lineFlags.read();
        // temporary read to allow breaks for alternative processes
        let totalNumLines = tokenLines.read().len();
        for lineNumber in 0..totalNumLines {
            // cloning because performance doesn't matter nearly as much here, and the
            // token lines are fairly short. It should also only keep one line at a time
            if lineNumber >= tokenLines.read().len() {  continue;  }  // stopping if an error is encountered
            let tokens = tokenLines.read()[lineNumber].clone();
            let mut lineText = String::new();
            for token in &tokens {
                // this would be referenced, but should be dropped by the line's ending
                lineText.push_str(&token.text);
            }
            HandleDefinitions(&tokens,
                              lineFlags,
                              outline,
                              &lineText, lineNumber
            );
        }

        // tracking the scope (functions = new scope; struct/enums = new scope; for/while = new scope)
        let tokenLinesRead = tokenLines.read();
        let mut rootNode = ScopeNode {
            children: vec![],
            name: "Root".to_string(),
            start: 0,
            end: tokenLinesRead.len().saturating_sub(1),
        };

        let mut jumps: Vec <Vec <usize>> = vec!();
        let mut linearized: Vec <Vec <usize>> = vec!();

        let mut currentScope: Vec <usize> = vec!();
        for (lineNumber, tokens) in tokenLinesRead.iter().enumerate() {
            HandleBracketsLayer(
                outline,
                &mut rootNode,
                &mut jumps,
                &mut linearized,
                &mut currentScope,
                tokens,
                lineNumber
            );
        }

        // dropping the memory for reading tokenLines
        // hopefully anything needing to write has a brief chance here
        // !!!!!!! Make sure there aren't any writes to tokenLines or it'll crash
        drop(tokenLinesRead);

        UpdateKeywordOutline(tokenLines, lineFlags, outline, &jumps, &rootNode);

        // updating the original
        let mut outlineWrite = outlineOriginal.write();
        outlineWrite.clear();
        while let Some(keyword) = outline.pop() {
            outlineWrite.push(keyword);
        } drop(outlineWrite);

        (rootNode, jumps, linearized)
    }
}

fn HandleBracketsLayer (
    outline: &mut [OutlineKeyword],
    rootNode: &mut ScopeNode,
    jumps: &mut Vec <Vec <usize>>,
    linearized: &mut Vec <Vec <usize>>,
    currentScope: &mut Vec <usize>,
    tokens: &[LuaTuple],
    lineNumber: usize
) {
    let mut bracketDepth = 0isize;
    // track the depth; if odd than based on the type of bracket add scope; if even do nothing (scope opened and closed on the same line)
    // use the same on functions to determine if the scope needs to continue or end on that line
    for (index, token) in tokens.iter().enumerate() {
        CalculateBracketDepth(
            tokens,
            &token.token,
            &mut bracketDepth,
            &token.text,
            index
        );

        // checking for something to define the name
        if matches!(token.token, TokenType::Keyword | TokenType::Object | TokenType::Function) {
            if !(VALID_NAMES_NEXT.contains(&token.text.trim()) || VALID_NAMES_TAKE.contains(&token.text.trim())) {
                continue;
            }

            let invalid = CheckForScopeName(
                tokens,
                rootNode,
                currentScope,
                linearized,
                &token.text,
                &mut bracketDepth,
                index,
                lineNumber
            );
            if invalid {  break;  }
        }
    }

    UpdateBracketDepth(outline, rootNode, currentScope, linearized, jumps, &mut bracketDepth, lineNumber);
}

fn CalculateBracketDepth (
    tokens: &[LuaTuple],
    token: &TokenType,
    bracketDepth: &mut isize,
    name: &String,
    index: usize,
) {
    let lastToken = {
        if index == 0 {  &LuaTuple { token: TokenType::Null, text: String::new() }  }
        else {  &tokens[index - 1]  }
    };
    if !(matches!(token, TokenType::Comment) || (lastToken.text == "\"" || lastToken.text == "'") && matches!(token, TokenType::String)) {
        // checking bracket depth
        if name == "{" {
            *bracketDepth += 1;
        } else if name == "}" {
            *bracketDepth -= 1;
        }
    }
}

fn CheckForScopeName (
    tokens: &[LuaTuple],
    rootNode: &mut ScopeNode,
    currentScope: &mut Vec <usize>,
    linearized: &mut Vec <Vec <usize>>,
    name: &String,
    bracketDepth: &mut isize,
    index: usize,
    lineNumber: usize
) -> bool {
    // checking the scope to see if ti continues or ends on the same line
    let mut brackDepth = 0isize;
    for (indx, token) in tokens.iter().enumerate() {
        let lastToken =
            if indx == 0 {  &LuaTuple { token: TokenType::Null, text: String::new() }  }
            else {  &tokens[indx - 1]  };
        if !(matches!(token.token, TokenType::Comment) || lastToken.text == "\"" && matches!(token.token, TokenType::String)) &&
            indx > index
        {
            if name == "{" {
                brackDepth += 1;
            } else if name == "}" {
                brackDepth -= 1;
            }
        }
    }

    UpdateScopes(
        tokens,
        rootNode,
        currentScope,
        linearized,
        name,
        bracketDepth,
        &mut brackDepth,
        index,
        lineNumber
    )
}

fn UpdateScopes (
    tokens: &[LuaTuple],
    rootNode: &mut ScopeNode,
    currentScope: &mut Vec <usize>,
    linearized: &mut Vec <Vec <usize>>,
    name: &str,
    bracketDepth: &mut isize,
    brackDepth: &mut isize,
    index: usize,
    lineNumber: usize
) -> bool {
    // check bracket-depth here so any overlapping marks are accounted for
    if *bracketDepth < 0 && *brackDepth > 0 {  // not pushing any jumps bc/ it'll be done later
        let mut scopeCopy = currentScope.clone();
        scopeCopy.reverse();
        rootNode.SetEnd(&mut scopeCopy, lineNumber);
        currentScope.pop();
    }

    // adding the new scope if necessary
    if *brackDepth > 0 {
        if VALID_NAMES_NEXT.contains(&name.trim()) {
            let nextName = tokens
                .get(index + 2)
                .unwrap_or( &LuaTuple { token: TokenType::Null, text: String::new() } )
                .text
                .clone();

            let mut scopeCopy = currentScope.clone();
            scopeCopy.reverse();
            let mut goodName = name.to_owned();
            goodName.push(' ');
            goodName.push_str(nextName.as_str());
            let newScope = rootNode.Push(&mut scopeCopy, goodName, lineNumber);
            currentScope.push(newScope);
            linearized.push(currentScope.clone());

            *bracketDepth = 0;
            return true;
        } else if VALID_NAMES_TAKE.contains(&name.trim()) {
            let mut scopeCopy = currentScope.clone();
            scopeCopy.reverse();
            let goodName = name.to_owned();
            let newScope = rootNode.Push(&mut scopeCopy, goodName, lineNumber);
            currentScope.push(newScope);
            linearized.push(currentScope.clone());

            *bracketDepth = 0;
            return true;
        }
    } false
}

fn UpdateBracketDepth (
    outline: &mut [OutlineKeyword],
    rootNode: &mut ScopeNode,
    currentScope: &mut Vec <usize>,
    linearized: &mut Vec <Vec <usize>>,
    jumps: &mut Vec <Vec <usize>>,
    bracketDepth: &mut isize,
    lineNumber: usize
) {
    // updating the scope based on the brackets
    if *bracketDepth > 0 {
        let mut scopeCopy = currentScope.clone();
        scopeCopy.reverse();
        let newScope = rootNode.Push(&mut scopeCopy, "{ ... }".to_string(), lineNumber);
        currentScope.push(newScope);
        jumps.push(currentScope.clone());
        linearized.push(currentScope.clone());
        OutlineKeyword::EditScopes(outline, currentScope, lineNumber);
    } else if *bracketDepth < 0 {
        jumps.push(currentScope.clone());
        OutlineKeyword::EditScopes(outline, currentScope, lineNumber);
        let mut scopeCopy = currentScope.clone();
        scopeCopy.reverse();
        rootNode.SetEnd(&mut scopeCopy, lineNumber);
        currentScope.pop();
    } else {
        jumps.push(currentScope.clone());
        OutlineKeyword::EditScopes(outline, currentScope, lineNumber);
    }
}

