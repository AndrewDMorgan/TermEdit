
pub mod Colors {
    use crate::TermRender::*;
    use crate::Tokens::*;
    //use ratatui::style::Color;

    // the types of color modes (some aren't supported by certain terminals...)
    #[derive(Debug, Default, PartialEq, Eq, Hash)]
    pub enum ColorTypes {
        #[default] TrueColor,
        PartialColor,
        BasicColor
    }
    
    #[derive(Debug)]
    pub struct ColorBindings <'a> {
        // for error text
        pub errorCol: ColorType,
        pub suggestion: ColorType,
        pub command: ColorType,
        pub default: ColorType,  // for default items (usually white)
        pub highlight: ColorType,

        // for syntax highlighting
        pub syntaxHighlighting: std::collections::HashMap<(&'a TokenType, &'a ColorTypes), ColorType>,
    }

    // the default is truecolor; other settings are available
    impl <'a> Default for ColorBindings <'a> {
        fn default() -> ColorBindings <'a> {
            ColorBindings {
                errorCol: ColorType::Red,
                suggestion: ColorType::White,
                command: ColorType::White,
                default: ColorType::White,
                highlight: ColorType::Yellow,

                syntaxHighlighting: std::collections::HashMap::from([
                    ((&TokenType::Bracket, &ColorTypes::BasicColor), ColorType::BrightBlue),
                    ((&TokenType::SquirlyBracket, &ColorTypes::BasicColor), ColorType::Magenta),
                    ((&TokenType::Parentheses, &ColorTypes::BasicColor), ColorType::Magenta),
                    ((&TokenType::Variable, &ColorTypes::BasicColor), ColorType::White),
                    ((&TokenType::Member, &ColorTypes::BasicColor), ColorType::BrightCyan),
                    ((&TokenType::Object, &ColorTypes::BasicColor), ColorType::BrightRed),
                    ((&TokenType::Function, &ColorTypes::BasicColor), ColorType::BrightMagenta),
                    ((&TokenType::Method, &ColorTypes::BasicColor), ColorType::BrightBlue),
                    ((&TokenType::Number, &ColorTypes::BasicColor), ColorType::BrightYellow),
                    ((&TokenType::Logic, &ColorTypes::BasicColor), ColorType::BrightYellow),
                    ((&TokenType::Math, &ColorTypes::BasicColor), ColorType::BrightYellow),
                    ((&TokenType::Assignment, &ColorTypes::BasicColor), ColorType::BrightBlue),
                    ((&TokenType::Endl, &ColorTypes::BasicColor), ColorType::White),
                    ((&TokenType::Macro, &ColorTypes::BasicColor), ColorType::BrightMagenta),
                    ((&TokenType::Const, &ColorTypes::BasicColor), ColorType::Cyan),
                    ((&TokenType::Barrow, &ColorTypes::BasicColor), ColorType::BrightGreen),
                    ((&TokenType::Lifetime, &ColorTypes::BasicColor), ColorType::BrightBlue),
                    ((&TokenType::String, &ColorTypes::BasicColor), ColorType::Yellow),
                    ((&TokenType::Comment, &ColorTypes::BasicColor), ColorType::Green),
                    ((&TokenType::CommentLong, &ColorTypes::BasicColor), ColorType::Green),
                    ((&TokenType::Null, &ColorTypes::BasicColor), ColorType::White),
                    ((&TokenType::Primitive, &ColorTypes::BasicColor), ColorType::BrightYellow),
                    ((&TokenType::Keyword, &ColorTypes::BasicColor), ColorType::BrightRed),
                    ((&TokenType::Unsafe, &ColorTypes::BasicColor), ColorType::BrightRed),

                    ((&TokenType::Bracket, &ColorTypes::PartialColor), ColorType::BrightBlue),
                    ((&TokenType::SquirlyBracket, &ColorTypes::PartialColor), ColorType::Magenta),
                    ((&TokenType::Parentheses, &ColorTypes::PartialColor), ColorType::Magenta),
                    ((&TokenType::Variable, &ColorTypes::PartialColor), ColorType::White),
                    ((&TokenType::Member, &ColorTypes::PartialColor), ColorType::BrightCyan),
                    ((&TokenType::Object, &ColorTypes::PartialColor), ColorType::BrightRed),
                    ((&TokenType::Function, &ColorTypes::PartialColor), ColorType::BrightMagenta),
                    ((&TokenType::Method, &ColorTypes::PartialColor), ColorType::BrightBlue),
                    ((&TokenType::Number, &ColorTypes::PartialColor), ColorType::BrightYellow),
                    ((&TokenType::Logic, &ColorTypes::PartialColor), ColorType::BrightYellow),
                    ((&TokenType::Math, &ColorTypes::PartialColor), ColorType::BrightYellow),
                    ((&TokenType::Assignment, &ColorTypes::PartialColor), ColorType::BrightBlue),
                    ((&TokenType::Endl, &ColorTypes::PartialColor), ColorType::White),
                    ((&TokenType::Macro, &ColorTypes::PartialColor), ColorType::BrightMagenta),
                    ((&TokenType::Const, &ColorTypes::PartialColor), ColorType::Cyan),
                    ((&TokenType::Barrow, &ColorTypes::PartialColor), ColorType::BrightGreen),
                    ((&TokenType::Lifetime, &ColorTypes::PartialColor), ColorType::BrightBlue),
                    ((&TokenType::String, &ColorTypes::PartialColor), ColorType::Yellow),
                    ((&TokenType::Comment, &ColorTypes::PartialColor), ColorType::Green),
                    ((&TokenType::CommentLong, &ColorTypes::PartialColor), ColorType::Green),
                    ((&TokenType::Null, &ColorTypes::PartialColor), ColorType::White),
                    ((&TokenType::Primitive, &ColorTypes::PartialColor), ColorType::BrightYellow),
                    ((&TokenType::Keyword, &ColorTypes::PartialColor), ColorType::BrightRed),
                    ((&TokenType::Unsafe, &ColorTypes::PartialColor), ColorType::BrightRed),

                    ((&TokenType::Bracket, &ColorTypes::TrueColor), ColorType::RGB(125, 180, 255)),
                    ((&TokenType::SquirlyBracket, &ColorTypes::TrueColor), ColorType::RGB(175, 50, 175)),
                    ((&TokenType::Parentheses, &ColorTypes::TrueColor), ColorType::RGB(175, 50, 175)),
                    ((&TokenType::Variable, &ColorTypes::TrueColor), ColorType::RGB(225, 225, 225)),
                    ((&TokenType::Member, &ColorTypes::TrueColor), ColorType::BrightCyan),
                    ((&TokenType::Object, &ColorTypes::TrueColor), ColorType::RGB(225, 145, 110)),
                    ((&TokenType::Function, &ColorTypes::TrueColor), ColorType::BrightMagenta),
                    ((&TokenType::Method, &ColorTypes::TrueColor), ColorType::RGB(125, 180, 255)),
                    ((&TokenType::Number, &ColorTypes::TrueColor), ColorType::BrightYellow),
                    ((&TokenType::Logic, &ColorTypes::TrueColor), ColorType::RGB(225, 225, 150)),
                    ((&TokenType::Math, &ColorTypes::TrueColor), ColorType::RGB(225, 225, 150)),
                    ((&TokenType::Assignment, &ColorTypes::TrueColor), ColorType::BrightBlue),
                    ((&TokenType::Endl, &ColorTypes::TrueColor), ColorType::RGB(225, 225, 225)),
                    ((&TokenType::Macro, &ColorTypes::TrueColor), ColorType::BrightMagenta),
                    ((&TokenType::Const, &ColorTypes::TrueColor), ColorType::Cyan),
                    ((&TokenType::Barrow, &ColorTypes::TrueColor), ColorType::RGB(225, 225, 150)),
                    ((&TokenType::Lifetime, &ColorTypes::TrueColor), ColorType::BrightBlue),  // make this a different rgb?
                    ((&TokenType::String, &ColorTypes::TrueColor), ColorType::Yellow),
                    ((&TokenType::Comment, &ColorTypes::TrueColor), ColorType::RGB(35, 150, 45)),
                    ((&TokenType::CommentLong, &ColorTypes::TrueColor), ColorType::RGB(35, 150, 45)),
                    ((&TokenType::Null, &ColorTypes::TrueColor), ColorType::RGB(225, 225, 225)),
                    ((&TokenType::Primitive, &ColorTypes::TrueColor), ColorType::BrightYellow),
                    ((&TokenType::Keyword, &ColorTypes::TrueColor), ColorType::BrightRed),
                    ((&TokenType::Unsafe, &ColorTypes::TrueColor), ColorType::BrightRed),  // make its background less standing out?
                ]),
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct ColorMode <'a> {
        pub colorType: ColorTypes,
        pub colorBindings: ColorBindings <'a>,
    }
}

