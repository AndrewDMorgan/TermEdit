keywords = {"true", "false"}

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
    local lastToken = ""
    local lastTokenType = "Null"
    local inComment = false
    -- going through the vector and parsing them
    for i, token in ipairs(stringTokens) do
        local nextToken = stringTokens[i + 1]
        local nextNextToken = stringTokens[i + 2]
        local tokenType = "Null"
        -- handling multi-token flags
        if token == "#" or inComment then
            tokenType = "Comment"
            inComment = true
        elseif token == "\"" or token == "'" then
            inString = not inString
            tokenType = "String"
        elseif inString then
            tokenType = "String"
        elseif token == " " then
            tokenType = "Null"
        else
            tokenType = ParseBasic(lastToken, nextToken, token, nextNextToken, i)
        end
        table.insert(parsedTokens, {tokenType, token})
        lastTokenType = tokenType
        lastToken = token
    end
    return parsedTokens
end
-- parses basic tokens like brackets
function ParseBasic (lastToken, nextToken, token, nextNextToken, i)
    if token == "(" or token == ")" then
        return "Parentheses"
    elseif token == "[" or token == "]" or token == "{" or token == "}" then
        return "Bracket"
    elseif Contains(keywords, token) then
        return "Keyword"
    elseif lastToken == "[" and nextToken == "]" then
        return "Method"
    elseif token == "=" then
        return "Assignment"
    elseif nextNextToken == "=" then
        if i == 1 then
            return "Variable"
        end
        return "Member"
    end
    return "Null"  -- no tokens were found yet
end

