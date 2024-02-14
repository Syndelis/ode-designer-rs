use std::{fmt::Display, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, anychar, space0},
    character::complete::digit1,
    combinator::map,
    combinator::recognize,
    multi::{many0, many0_count, many1},
    sequence::{pair, preceded},
    IResult,
};

#[derive(Debug)]
pub struct Format(Vec<FormatPart>);

impl Format {
    pub fn default_with_name(fn_name: &str) -> Self {
        Self(
            vec![
                FormatPart::Static(fn_name.to_owned()),
                FormatPart::Static("(".to_owned()),
                FormatPart::Dynamic(ArgumentSpecifier::All { separator: ',' }),
                FormatPart::Static(")".to_owned()),
            ]
        )
    }

    pub fn format_args<'a, T: Display + 'a>(&self, args: impl Iterator<Item=&'a T>) -> String {
        let mut str_buf = [0; 4];
        let display_args: Vec<_> = args.map(ToString::to_string).collect();

        self
            .0
            .iter()
            .map(|part| {
                match part {
                    FormatPart::Static(s) => s.clone(),
                    FormatPart::Dynamic(arg_spec) => match arg_spec {
                        ArgumentSpecifier::Indexed(idx) =>
                            display_args
                                .get(*idx - 1)
                                .map(Clone::clone)
                                .unwrap_or_else(|| "_".to_string()),
                        ArgumentSpecifier::Named(_) => todo!("Named parameters are still not supported"),
                        ArgumentSpecifier::All { separator } => {
                            display_args.join(separator.encode_utf8(&mut str_buf))
                        },
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl FromStr for Format {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_format(s)
            .map(|(_rest, parts)| Self(parts))
            .map_err(|err| err.to_owned())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FormatPart {
    Static(String),
    Dynamic(ArgumentSpecifier),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgumentSpecifier {
    Indexed(usize),
    Named(String),
    All { separator: char },
}

fn parse_ident(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parse_dollar_preceded_element(input: &str) -> IResult<&str, FormatPart> {
    alt((
        map(digit1, |numb: &str| {
            FormatPart::Dynamic(ArgumentSpecifier::Indexed(numb.parse().unwrap()))
        }),
        map(parse_ident, |ident: &str| {
            FormatPart::Dynamic(ArgumentSpecifier::Named(ident.to_owned()))
        }),
        map(pair(tag("@"), anychar), |(_at, c)| {
            FormatPart::Dynamic(ArgumentSpecifier::All { separator: c })
        }),
        map(many0(is_not("$ ")), |vc| {
            let mut s = String::from("$");
            vc.into_iter().collect_into(&mut s);
            FormatPart::Static(s)
        }),
    ))(input)
}

fn parse_format(input: &str) -> IResult<&str, Vec<FormatPart>> {
    fn parse_element(input: &str) -> IResult<&str, FormatPart> {
        preceded(
            space0,
            alt((
                preceded(tag("$"), parse_dollar_preceded_element),
                map(many1(is_not("$ ")), |st| {
                    FormatPart::Static(st.into_iter().collect())
                }),
            )),
        )(input)
    }

    many0(parse_element)(input)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    use FormatPart::{
        Static as S,
        Dynamic as D,
    };

    use ArgumentSpecifier::{
        Indexed as I,
        Named as N,
        All,
    };

    #[rstest]
    #[case(
        "   $1    *    $2",
        &[
            D(I(1)),
            S("*".to_string()),
            D(I(2)),
        ]
    )]
    #[case(
        "cos($1)",
        &[
            S("cos(".to_string()),
            D(I(1)),
            S(")".to_string()),
        ]
    )]
    #[case(
        "Sum = $@+",
        &[
            S("Sum".to_string()),
            S("=".to_string()),
            D(All { separator: '+' }),
        ]
    )]
    #[case(
        "$x ^ $y",
        &[
            D(N("x".to_string())),
            S("^".to_string()),
            D(N("y".to_string())),
        ]
    )]
    #[case(
        "π$r²",
        &[
            S("π".to_string()),
            D(N("r".to_string())),
            S("²".to_string()),
        ]
    )]
    fn test_format_parsing(
        #[case] input: &str,
        #[case] expected_format: &[FormatPart]
    ) {
        let (rest, parsed) = parse_format(input).expect("Parsing failed!");
        
        assert!(rest.is_empty(), "There was a string remainder: '{rest}'");

        assert_eq!(parsed, expected_format);
    }

    #[rstest]
    #[case(
        Format(vec![
            S("cos(".to_string()),
            D(I(1)),
            S(")".to_string()),
        ]),
        &["x"],
        "cos( x )",
    )]
    #[case(
        Format(vec![
            S("cos(".to_string()),
            D(I(1)),
            S(")".to_string()),
        ]),
        &[] as &[&str],
        "cos( _ )",
    )]
    #[case(
        Format(vec![
            D(I(1)),
            S("*".to_string()),
            D(I(2)),
        ]),
        &[10, 20],
        "10 * 20",
    )]
    #[case(
        Format(vec![
            S("Sum".to_string()),
            S("=".to_string()),
            D(All { separator: '+' }),
        ]),
        &['A', 'B', 'C', 'D'],
        "Sum = A+B+C+D",
    )]
    fn test_format_args<T: Display>(
        #[case] format: Format,
        #[case] args: &'static [T],
        #[case] expected_formatted_text: &'static str,
    ) {
        let res = format.format_args(args.iter());
        assert_eq!(res, expected_formatted_text);
    }
}