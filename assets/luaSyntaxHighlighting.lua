keywords = {"if", "for", "while", "in", "else", "break", "elseif",
            "return", "function", "local", "do", "then", "end", "nil",
            "or", "and", "not"}
mathLogicTokens = {"=", "<", ">", "~", "-", "+", "/", "*"}
logicTokens = {"=", "<", ">", "~"}
mathTokens = {"-", "+", "/", "*"}


-- checks if a value is in an array
function Contains (array, query)
    for index = 1, #array do
        if array[index] == query then
            return true
end end end


-- takes in a vector of strings
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
        if token == "\"" or token == "'" then
            inString = not inString
            tokenType = "String"
        elseif token == "-" and (lastToken == "-" or nextToken == "-") then
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
            tokenType = ParseTokenType(lastTokenType, lastToken, nextToken, token)
        end

        table.insert(parsedTokens, {tokenType, token})
        lastTokenType = tokenType
        lastToken = token
    end

    return parsedTokens
end

-- parses basic tokens like brackets
function ParseBasic (lastTokenType, lastToken, nextToken, token)
    if token == "(" or token == ")" then
        return "Parentheses"
    elseif token == "[" or token == "]" or token == "{" or token == "}" then
        return "Bracket"
    elseif token == "=" and not Contains(mathLogicTokens, lastToken) and nextToken ~= "=" then
        return "Assignment"
    end

    return "Null"  -- no tokens were found yet
end

-- handles various extras
function ParseExtras (lastTokenType, lastToken, nextToken, token)
    if token == ">" or token == "<" then
        return "Logic"
    elseif token == "=" and Contains(logicTokens, lastToken) then
        return "Logic"
    elseif Contains(logicTokens, token) and nextToken == "=" then
        return "Logic"
    elseif Contains(mathTokens, token) then
        return "Math"
    elseif token == "=" and (nextToken == "=" or Contains(mathTokens, lastToken)) then
        return "Math"
    elseif tonumber(string.sub(token, 1, 1)) ~= nil then
        return "Number"
    end
    return "Null"
end

-- does the more complex parts of token-parsing (not multi-token flags)
function ParseTokenType (lastTokenType, lastToken, nextToken, token)
    -- parsing the basic tokens
    local tokenType = ParseBasic(lastTokenType, lastToken, nextToken, token)
    if tokenType ~= "Null" then
        return tokenType
    end

    local startingCharacter = string.sub(token, 1, 1)
    if Contains(keywords, token) then
        return "Keyword"
    elseif Contains(mathLogicTokens, token) then
        return ParseExtras(lastTokenType, lastToken, nextToken, token)
    elseif string.upper(startingCharacter) == startingCharacter then
        return "Function"
    end
end

