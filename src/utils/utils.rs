use std::{fmt::Display, str::FromStr};

use num_traits::{Num, Unsigned};
use regex::Regex;

use crate::debugger::terminal_commands::TerminalCommandErrors;

pub fn print_vec<T: Display>(vec: &Vec<T>) -> String {
    let mut s = String::new();

    for elem in vec {
        s.push_str(&elem.to_string());
        s.push_str(",");
    }

    s
}



#[derive(Debug, PartialEq)]
pub struct ParsingError;

fn parse_hex<T: Num>(str: &str) -> Result<T, ParsingError> {
    let hex_rust = Regex::new(r"^0x([0-9a-fA-F]*)$").unwrap();

    let Some(caps) = hex_rust.captures(str) else {
        return Err(ParsingError);
    };

    let hex_str = caps.get(1).unwrap().as_str();
    let Ok(res) = T::from_str_radix(hex_str, 16) else {
        return Err(ParsingError);
    };
    
    return Ok(res);
}

pub fn try_parse_num<T: Num + FromStr>(str: &str) -> Result<T, ParsingError> {
    if let Ok(parsed_value) = str.parse() {
        return Ok(parsed_value);
    } 

    parse_hex::<T>(str)
}


#[cfg(test)]
mod util_tests {
    use rstest::rstest;

    use crate::utils::utils::parse_hex;
        use crate::utils::utils::ParsingError;

    #[rstest]
    #[case("0x123", 0x123)]
    #[case("0xA23", 0xA23)]
    #[case("0xABCDEF12345678", 0xABCDEF12345678)]
    fn test_parses_hex_happy_case(#[case] str: &str, #[case] expected: u64) {
        assert_eq!(expected, parse_hex(str).unwrap());
    }

    #[rstest]
    #[case("21345")]
    #[case("-0x1234")]
    fn test_parses_hex_parsing_error(#[case] str: &str) {
        assert_eq!(ParsingError, parse_hex::<u32>(str).unwrap_err());
    }


}
