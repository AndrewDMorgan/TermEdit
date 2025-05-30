
use crate::TermRender::*;
use crate::Tokens::*;
//use ratatui::style::Color;

// the types of color modes (some aren't supported by certain terminals...)
#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorTypes {
    #[default] True,
    Partial,
    Basic
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
                ((&TokenType::Bracket, &ColorTypes::Basic), ColorType::BrightBlue),
                ((&TokenType::SquirlyBracket, &ColorTypes::Basic), ColorType::Magenta),
                ((&TokenType::Parentheses, &ColorTypes::Basic), ColorType::Magenta),
                ((&TokenType::Variable, &ColorTypes::Basic), ColorType::White),
                ((&TokenType::Member, &ColorTypes::Basic), ColorType::BrightCyan),
                ((&TokenType::Object, &ColorTypes::Basic), ColorType::BrightRed),
                ((&TokenType::Function, &ColorTypes::Basic), ColorType::BrightMagenta),
                ((&TokenType::Method, &ColorTypes::Basic), ColorType::BrightBlue),
                ((&TokenType::Number, &ColorTypes::Basic), ColorType::BrightYellow),
                ((&TokenType::Logic, &ColorTypes::Basic), ColorType::BrightYellow),
                ((&TokenType::Math, &ColorTypes::Basic), ColorType::BrightYellow),
                ((&TokenType::Assignment, &ColorTypes::Basic), ColorType::BrightBlue),
                ((&TokenType::Endl, &ColorTypes::Basic), ColorType::White),
                ((&TokenType::Macro, &ColorTypes::Basic), ColorType::BrightMagenta),
                ((&TokenType::Const, &ColorTypes::Basic), ColorType::Cyan),
                ((&TokenType::Barrow, &ColorTypes::Basic), ColorType::BrightGreen),
                ((&TokenType::Lifetime, &ColorTypes::Basic), ColorType::BrightBlue),
                ((&TokenType::String, &ColorTypes::Basic), ColorType::Yellow),
                ((&TokenType::Comment, &ColorTypes::Basic), ColorType::Green),
                ((&TokenType::CommentLong, &ColorTypes::Basic), ColorType::Green),
                ((&TokenType::Null, &ColorTypes::Basic), ColorType::White),
                ((&TokenType::Primitive, &ColorTypes::Basic), ColorType::BrightYellow),
                ((&TokenType::Keyword, &ColorTypes::Basic), ColorType::BrightRed),
                ((&TokenType::Unsafe, &ColorTypes::Basic), ColorType::BrightRed),

                ((&TokenType::Bracket, &ColorTypes::Partial), ColorType::BrightBlue),
                ((&TokenType::SquirlyBracket, &ColorTypes::Partial), ColorType::Magenta),
                ((&TokenType::Parentheses, &ColorTypes::Partial), ColorType::Magenta),
                ((&TokenType::Variable, &ColorTypes::Partial), ColorType::White),
                ((&TokenType::Member, &ColorTypes::Partial), ColorType::BrightCyan),
                ((&TokenType::Object, &ColorTypes::Partial), ColorType::BrightRed),
                ((&TokenType::Function, &ColorTypes::Partial), ColorType::BrightMagenta),
                ((&TokenType::Method, &ColorTypes::Partial), ColorType::BrightBlue),
                ((&TokenType::Number, &ColorTypes::Partial), ColorType::BrightYellow),
                ((&TokenType::Logic, &ColorTypes::Partial), ColorType::BrightYellow),
                ((&TokenType::Math, &ColorTypes::Partial), ColorType::BrightYellow),
                ((&TokenType::Assignment, &ColorTypes::Partial), ColorType::BrightBlue),
                ((&TokenType::Endl, &ColorTypes::Partial), ColorType::White),
                ((&TokenType::Macro, &ColorTypes::Partial), ColorType::BrightMagenta),
                ((&TokenType::Const, &ColorTypes::Partial), ColorType::Cyan),
                ((&TokenType::Barrow, &ColorTypes::Partial), ColorType::BrightGreen),
                ((&TokenType::Lifetime, &ColorTypes::Partial), ColorType::BrightBlue),
                ((&TokenType::String, &ColorTypes::Partial), ColorType::Yellow),
                ((&TokenType::Comment, &ColorTypes::Partial), ColorType::Green),
                ((&TokenType::CommentLong, &ColorTypes::Partial), ColorType::Green),
                ((&TokenType::Null, &ColorTypes::Partial), ColorType::White),
                ((&TokenType::Primitive, &ColorTypes::Partial), ColorType::BrightYellow),
                ((&TokenType::Keyword, &ColorTypes::Partial), ColorType::BrightRed),
                ((&TokenType::Unsafe, &ColorTypes::Partial), ColorType::BrightRed),

                ((&TokenType::Bracket, &ColorTypes::True), ColorType::Rgb(125, 180, 255)),
                ((&TokenType::SquirlyBracket, &ColorTypes::True), ColorType::Rgb(175, 50, 175)),
                ((&TokenType::Parentheses, &ColorTypes::True), ColorType::Rgb(175, 50, 175)),
                ((&TokenType::Variable, &ColorTypes::True), ColorType::Rgb(225, 225, 225)),
                ((&TokenType::Member, &ColorTypes::True), ColorType::BrightCyan),
                ((&TokenType::Object, &ColorTypes::True), ColorType::Rgb(225, 145, 110)),
                ((&TokenType::Function, &ColorTypes::True), ColorType::BrightMagenta),
                ((&TokenType::Method, &ColorTypes::True), ColorType::Rgb(125, 180, 255)),
                ((&TokenType::Number, &ColorTypes::True), ColorType::BrightYellow),
                ((&TokenType::Logic, &ColorTypes::True), ColorType::Rgb(225, 225, 150)),
                ((&TokenType::Math, &ColorTypes::True), ColorType::Rgb(225, 225, 150)),
                ((&TokenType::Assignment, &ColorTypes::True), ColorType::BrightBlue),
                ((&TokenType::Endl, &ColorTypes::True), ColorType::Rgb(225, 225, 225)),
                ((&TokenType::Macro, &ColorTypes::True), ColorType::BrightMagenta),
                ((&TokenType::Const, &ColorTypes::True), ColorType::Cyan),
                ((&TokenType::Barrow, &ColorTypes::True), ColorType::Rgb(225, 225, 150)),
                ((&TokenType::Lifetime, &ColorTypes::True), ColorType::BrightBlue),  // make this a different rgb?
                ((&TokenType::String, &ColorTypes::True), ColorType::Yellow),
                ((&TokenType::Comment, &ColorTypes::True), ColorType::Rgb(35, 150, 45)),
                ((&TokenType::CommentLong, &ColorTypes::True), ColorType::Rgb(35, 150, 45)),
                ((&TokenType::Null, &ColorTypes::True), ColorType::Rgb(225, 225, 225)),
                ((&TokenType::Primitive, &ColorTypes::True), ColorType::BrightYellow),
                ((&TokenType::Keyword, &ColorTypes::True), ColorType::BrightRed),
                ((&TokenType::Unsafe, &ColorTypes::True), ColorType::BrightRed),  // make its background less standing out?
            ]),
        }
    }
}

#[derive(Debug, Default)]
pub struct ColorMode <'a> {
    pub colorType: ColorTypes,
    pub colorBindings: ColorBindings <'a>,
}

