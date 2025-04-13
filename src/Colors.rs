
pub mod Colors {
    use crate::Tokens::*;
    use ratatui::style::Color;

    // the types of color modes (some aren't supported by certain terminals...)
    #[derive(Debug, Default)]
    pub enum ColorTypes {
        #[default] TrueColor,
        PartialColor,
        BasicColor
    }

    #[derive(Debug)]
    pub struct ColorBindings {
        // for error text
        pub errorCol: Color,
        pub suggestion: Color,
        pub command: Color,
        pub default: Color,  // for default items (usually white)
        pub highlight: Color,

        // for syntax highlighting
        pub syntaxHighlighting: std::collections::HashMap<TokenType, Color>,
    }

    // the default is truecolor; other settings are available
    impl Default for ColorBindings {
        fn default() -> ColorBindings {
            ColorBindings {
                errorCol: Color::Red,
                suggestion: Color::White,
                command: Color::White,
                default: Color::White,
                highlight: Color::Yellow,

                syntaxHighlighting: std::collections::HashMap::from([
                    (TokenType::Bracket, Color::LightBlue),
                    (TokenType::SquirlyBracket, Color::Magenta),
                    (TokenType::Parentheses, Color::Magenta),
                    (TokenType::Variable, Color::White),
                    (TokenType::Member, Color::LightCyan),
                    (TokenType::Object, Color::LightRed),
                    (TokenType::Function, Color::LightMagenta),
                    (TokenType::Method, Color::LightBlue),
                    (TokenType::Number, Color::LightYellow),
                    (TokenType::Logic, Color::LightYellow),
                    (TokenType::Math, Color::LightYellow),
                    (TokenType::Assignment, Color::LightBlue),
                    (TokenType::Endl, Color::White),
                    (TokenType::Macro, Color::LightMagenta),
                    (TokenType::Const, Color::Cyan),
                    (TokenType::Barrow, Color::LightGreen),
                    (TokenType::Lifetime, Color::LightBlue),
                    (TokenType::String, Color::Yellow),
                    (TokenType::Comment, Color::Rgb(35, 150, 45)),
                    (TokenType::CommentLong, Color::Rgb(35, 150, 45)),
                    (TokenType::Null, Color::White),
                    (TokenType::Primitive, Color::LightYellow),
                    (TokenType::Keyword, Color::LightRed),
                    (TokenType::Unsafe, Color::LightRed),
                ]),
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct ColorMode {
        pub colorType: ColorTypes,
        pub colorBindings: ColorBindings,
    }
}

