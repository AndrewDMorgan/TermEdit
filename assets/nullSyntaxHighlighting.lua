

-- takes in a vector of strings
function GetTokens (stringTokens)
    local parsedTokens = {}
    -- going through the vector and parsing them
    for _, token in ipairs(stringTokens) do
        local tokenType = "Null"  -- no highlighting
        table.insert(parsedTokens, {tokenType, token})
    end

    return parsedTokens
end


