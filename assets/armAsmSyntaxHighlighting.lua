keywords = {"MOV", "ADD", "SUB", "CMP", "INC", "DEC", "JMP",
            "JE", "JNE", "CALL", "RET", "LODSB", "STOSB",
            "MOVSB", "PUSH", "POP", "TIMES", "HLT", "CLI",
            "DW", "INT", "ORG", "BITS", "$", "BYTE"
}
registers = {"AX", "BX", "CX", "DX", "SI", "DI", "BP", "SP",
             "AH", "AL", "BH", "BL", "CH", "CL", "DH", "DL",
             "CS", "DS", "ES", "SS", "IP", "FLAGS", "DB"
}

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
        if (token == "\"" or token == "'") and not inComment then
            inString = not inString
            tokenType = "String"
        elseif (token == ";" or token == "@" or token == "/" and nextToken == "/") and not inString then
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

-- does the more complex parts of token-parsing (not multi-token flags)
function ParseTokenType (lastTokenType, lastToken, nextToken, token)
    local upperToken = string.upper(token)
    local sub = string.sub(token, 1, 1)
    if tonumber(sub) ~= nil or tonumber(string.sub(token, 2, 2)) ~= nil and sub == "#" then
        return "Number"
    elseif Contains(keywords, upperToken) then
        return "Keyword"
    elseif string.sub(token, 1, 2) == "0x" or string.sub(token, 1, 3) == "#0x" then
        return "Member"
    elseif Contains(registers, upperToken) or Contains(registers, string.sub(upperToken, 2)) then
        return "Assignment"
    elseif token == "[" or token == "]" then
        return "Parentheses"
    elseif nextToken == ":" or lastToken == "." or token == "." then
        return "Function"  -- header
    elseif upperToken == token then
        return "Const"  -- instruction type
    end
end

