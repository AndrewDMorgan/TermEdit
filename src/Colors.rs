
pub mod Colors {
    use crate::Tokens::*;
    use ratatui::style::Color;

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
        pub errorCol: Color,
        pub suggestion: Color,
        pub command: Color,
        pub default: Color,  // for default items (usually white)
        pub highlight: Color,

        // for syntax highlighting
        pub syntaxHighlighting: std::collections::HashMap<(&'a TokenType, &'a ColorTypes), Color>,
    }

    // the default is truecolor; other settings are available
    impl <'a> Default for ColorBindings <'a> {
        fn default() -> ColorBindings <'a> {
            ColorBindings {
                errorCol: Color::Red,
                suggestion: Color::White,
                command: Color::White,
                default: Color::White,
                highlight: Color::Yellow,

                syntaxHighlighting: std::collections::HashMap::from([
                    ((&TokenType::Bracket, &ColorTypes::BasicColor), Color::LightBlue),
                    ((&TokenType::SquirlyBracket, &ColorTypes::BasicColor), Color::Magenta),
                    ((&TokenType::Parentheses, &ColorTypes::BasicColor), Color::Magenta),
                    ((&TokenType::Variable, &ColorTypes::BasicColor), Color::White),
                    ((&TokenType::Member, &ColorTypes::BasicColor), Color::LightCyan),
                    ((&TokenType::Object, &ColorTypes::BasicColor), Color::LightRed),
                    ((&TokenType::Function, &ColorTypes::BasicColor), Color::LightMagenta),
                    ((&TokenType::Method, &ColorTypes::BasicColor), Color::LightBlue),
                    ((&TokenType::Number, &ColorTypes::BasicColor), Color::LightYellow),
                    ((&TokenType::Logic, &ColorTypes::BasicColor), Color::LightYellow),
                    ((&TokenType::Math, &ColorTypes::BasicColor), Color::LightYellow),
                    ((&TokenType::Assignment, &ColorTypes::BasicColor), Color::LightBlue),
                    ((&TokenType::Endl, &ColorTypes::BasicColor), Color::White),
                    ((&TokenType::Macro, &ColorTypes::BasicColor), Color::LightMagenta),
                    ((&TokenType::Const, &ColorTypes::BasicColor), Color::Cyan),
                    ((&TokenType::Barrow, &ColorTypes::BasicColor), Color::LightGreen),
                    ((&TokenType::Lifetime, &ColorTypes::BasicColor), Color::LightBlue),
                    ((&TokenType::String, &ColorTypes::BasicColor), Color::Yellow),
                    ((&TokenType::Comment, &ColorTypes::BasicColor), Color::Green),
                    ((&TokenType::CommentLong, &ColorTypes::BasicColor), Color::Green),
                    ((&TokenType::Null, &ColorTypes::BasicColor), Color::White),
                    ((&TokenType::Primitive, &ColorTypes::BasicColor), Color::LightYellow),
                    ((&TokenType::Keyword, &ColorTypes::BasicColor), Color::LightRed),
                    ((&TokenType::Unsafe, &ColorTypes::BasicColor), Color::LightRed),

                    ((&TokenType::Bracket, &ColorTypes::PartialColor), Color::LightBlue),
                    ((&TokenType::SquirlyBracket, &ColorTypes::PartialColor), Color::Magenta),
                    ((&TokenType::Parentheses, &ColorTypes::PartialColor), Color::Magenta),
                    ((&TokenType::Variable, &ColorTypes::PartialColor), Color::White),
                    ((&TokenType::Member, &ColorTypes::PartialColor), Color::LightCyan),
                    ((&TokenType::Object, &ColorTypes::PartialColor), Color::LightRed),
                    ((&TokenType::Function, &ColorTypes::PartialColor), Color::LightMagenta),
                    ((&TokenType::Method, &ColorTypes::PartialColor), Color::LightBlue),
                    ((&TokenType::Number, &ColorTypes::PartialColor), Color::LightYellow),
                    ((&TokenType::Logic, &ColorTypes::PartialColor), Color::LightYellow),
                    ((&TokenType::Math, &ColorTypes::PartialColor), Color::LightYellow),
                    ((&TokenType::Assignment, &ColorTypes::PartialColor), Color::LightBlue),
                    ((&TokenType::Endl, &ColorTypes::PartialColor), Color::White),
                    ((&TokenType::Macro, &ColorTypes::PartialColor), Color::LightMagenta),
                    ((&TokenType::Const, &ColorTypes::PartialColor), Color::Cyan),
                    ((&TokenType::Barrow, &ColorTypes::PartialColor), Color::LightGreen),
                    ((&TokenType::Lifetime, &ColorTypes::PartialColor), Color::LightBlue),
                    ((&TokenType::String, &ColorTypes::PartialColor), Color::Yellow),
                    ((&TokenType::Comment, &ColorTypes::PartialColor), Color::Green),
                    ((&TokenType::CommentLong, &ColorTypes::PartialColor), Color::Green),
                    ((&TokenType::Null, &ColorTypes::PartialColor), Color::White),
                    ((&TokenType::Primitive, &ColorTypes::PartialColor), Color::LightYellow),
                    ((&TokenType::Keyword, &ColorTypes::PartialColor), Color::LightRed),
                    ((&TokenType::Unsafe, &ColorTypes::PartialColor), Color::LightRed),

                    ((&TokenType::Bracket, &ColorTypes::TrueColor), Color::Rgb(125, 180, 255)),
                    ((&TokenType::SquirlyBracket, &ColorTypes::TrueColor), Color::Rgb(175, 50, 175)),
                    ((&TokenType::Parentheses, &ColorTypes::TrueColor), Color::Rgb(175, 50, 175)),
                    ((&TokenType::Variable, &ColorTypes::TrueColor), Color::Rgb(225, 225, 225)),
                    ((&TokenType::Member, &ColorTypes::TrueColor), Color::LightCyan),
                    ((&TokenType::Object, &ColorTypes::TrueColor), Color::Rgb(225, 145, 110)),
                    ((&TokenType::Function, &ColorTypes::TrueColor), Color::LightMagenta),
                    ((&TokenType::Method, &ColorTypes::TrueColor), Color::Rgb(125, 180, 255)),
                    ((&TokenType::Number, &ColorTypes::TrueColor), Color::LightYellow),
                    ((&TokenType::Logic, &ColorTypes::TrueColor), Color::Rgb(225, 225, 150)),
                    ((&TokenType::Math, &ColorTypes::TrueColor), Color::Rgb(225, 225, 150)),
                    ((&TokenType::Assignment, &ColorTypes::TrueColor), Color::LightBlue),
                    ((&TokenType::Endl, &ColorTypes::TrueColor), Color::Rgb(225, 225, 225)),
                    ((&TokenType::Macro, &ColorTypes::TrueColor), Color::LightMagenta),
                    ((&TokenType::Const, &ColorTypes::TrueColor), Color::Cyan),
                    ((&TokenType::Barrow, &ColorTypes::TrueColor), Color::Rgb(225, 225, 150)),
                    ((&TokenType::Lifetime, &ColorTypes::TrueColor), Color::LightBlue),  // make this a different rgb?
                    ((&TokenType::String, &ColorTypes::TrueColor), Color::Yellow),
                    ((&TokenType::Comment, &ColorTypes::TrueColor), Color::Rgb(35, 150, 45)),
                    ((&TokenType::CommentLong, &ColorTypes::TrueColor), Color::Rgb(35, 150, 45)),
                    ((&TokenType::Null, &ColorTypes::TrueColor), Color::Rgb(225, 225, 225)),
                    ((&TokenType::Primitive, &ColorTypes::TrueColor), Color::LightYellow),
                    ((&TokenType::Keyword, &ColorTypes::TrueColor), Color::LightRed),
                    ((&TokenType::Unsafe, &ColorTypes::TrueColor), Color::LightRed),  // make its background less standing out?
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

