
keywords = {"if", "for", "while", "in", "else", "break", "elif", "as", "from",
            "return", "continue", "import", "None", "async", "await", "is", "with"}
primitives = {"int", "float", "double", "String", "tuple", "list", "True", "False"}
objects = {"class", "self"}
mathLogicTokens = {"=", "<", ">", "!", "-", "+", "/", "*", "and", "or"}
logicTokens = {"=", "<", ">", "!"}
mathTokens = {"-", "+", "/", "*", "and", "or"}

-- checks if a value is in an array
function Contains (array, query)
    for index = 1, #array do
        if array[index] == query then
            return true
end end end


-- takes in a vector of strings   (GetTokens is the interfaced function w/ Rust)
function GetTokens (stringTokens)
    local parsedTokens = {}

    local inString = false;
    local inComment = false;

    local lastToken = ""
    local lastTokenType = "Null"

    -- going through the vector and parsing them
    for i, token in ipairs(stringTokens) do
        local nextToken = stringTokens[i + 1]
        local tokenType = "Null"

        -- handling multi-token flags
        if token == "\"" then
            inString = not inString
            tokenType = "String"
        elseif token == "#" then
            inComment = true
            tokenType = "Comment"
        -- finding the token type
        elseif inString then
            tokenType = "String"
        elseif inComment then
            tokenType = "Comment"
        elseif token == " " then
            tokenType = "Null"
        else
            tokenType = ParseTokenType(lastTokenType, lastToken, nextToken, stringTokens[i + 2], token)
        end

        table.insert(parsedTokens, {tokenType, token})
        lastTokenType = tokenType
        lastToken = token
    end

    return parsedTokens
end

-- handles various extras
function ParseExtras (lastTokenType, lastToken, nextToken, token)
    if token == ">" or token == "<" or token == "!" then
        return "Logic"
    elseif token == "=" and Contains(logicTokens, lastToken) then
        return "Logic"
    elseif token == "&" and (nextToken == "&" or lastToken == "&") or token == "|" then
        return "Logic"
    elseif token == "&" then
        return "Barrow"
    elseif Contains(mathTokens, token) then
        return "Math"
    elseif token == "=" and (nextToken == "=" or Contains(mathTokens, lastToken)) then
        return "Math"
    elseif tonumber(string.sub(token, 1, 1)) ~= nil then
        return "Number"
    end

    return "Null"
end

-- checking for unsafe code
function Unchecked (token)
    if token == "unsafe" or token == "from_raw" then
        return "Unsafe"
    end

    local splitText = {}
    for str in string.gmatch(token, '([^_]+)') do
        table.insert(splitText, str)
    end
    if #splitText == 2 then
        if splitText[2] == "unchecked" then
            return "Unsafe"
        end
    end

    return "Null"
end

-- parses basic tokens like brackets
function ParseBasic (lastTokenType, lastToken, nextToken, token)
    if token == "(" or token == ")" then
        return "Parentheses"
    elseif token == "[" or token == "]" then
        return "Bracket"
    elseif token == "{" or token == "}" then
        return "SquirlyBracket"
    elseif token == "=" and not Contains(mathLogicTokens, lastToken) and nextToken ~= "=" then
        return "Assignment"
    elseif token == "def" then
        return "Function"
    end

    return Unchecked(token)
end

-- does the more complex parts of token-parsing (not multi-token flags)
function ParseTokenType (lastTokenType, lastToken, nextToken, nextNextToken, token)
    -- parsing the basic tokens
    local tokenType = ParseBasic(lastTokenType, lastToken, nextToken, token)
    if tokenType ~= "Null" then
        return tokenType
    end

    -- this needs the macros to be calculated but not the members, methods and objects
    tokenType = ParseExtras(lastTokenType, lastToken, nextToken, token)

    -- checking keywords & stuff
    if tokenType ~= "Null" then
        return tokenType
    end if Contains(keywords, token) then
        return "Keyword"
    elseif Contains(primitives, token) then
        return "Primitive"
    elseif Contains(objects, token) then
        return "Object"
    elseif token == ":" then
        return "Endl"
    elseif lastToken == "." then
        return CalculateMember(lastTokenType, lastToken, nextToken, token)
    else
        return ComplexTokens(lastTokenType, lastToken, nextToken, nextNextToken, token)
end end

-- calculating more complex tokens
function ComplexTokens (lastTokenType, lastToken, nextToken, nextNextToken, token)
    if token == "'" then
        return "String"
    elseif lastToken == "'" and nextToken == "'" then
        return "String"
    elseif string.upper(token) == token then
        return "Const"
    elseif string.upper(string.sub(token, 1, 1)) == string.sub(token, 1, 1) then
        return "Function"
    end

    return "Null"
end

-- calculating members/methods
function CalculateMember (lastTokenType, lastToken, nextToken, token)
    -- checking for a method
    local startingCharacter = string.sub(token, 1, 1)
    if string.upper(token) == token then
        return "Const"
    elseif string.upper(startingCharacter) == startingCharacter then
        return "Method"
    end

    return "Member"
end

