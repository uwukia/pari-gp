use std::{fmt, ops::Range};

use annotate_snippets::{Level, Snippet, AnnotationKind, Renderer};

pub trait FromGp: Sized {
    fn try_from<'s>(input: &'s str) -> Result<(Self, &'s str), ParseError<'s>>;
    fn name() -> &'static str;
}

pub trait IntoGp {
    fn into(&self) -> String;
}

pub struct ParseError<'s> {
    error: Box<dyn fmt::Display>,
    input: &'s str,
    name: &'static str,
    location: Option<Range<usize>>
}

impl<'s> ParseError<'s> {
    pub fn new<T>(error: T, input: &'s str, name: &'static str, mut location: Range<usize>) -> Self
        where T: fmt::Display + 'static
    {
        location.start = location.start.clamp(0, input.len());
        location.end   = location.end.clamp(location.start, input.len());

        Self {
            error: Box::new(error),
            input, name,
            location: (!location.is_empty()).then_some(location)
        }
    }

    pub fn without_location<T>(error: T, input: &'s str, name: &'static str) -> Self
        where T: fmt::Display + 'static
    {
        Self { error: Box::new(error), input, name, location: None }
    }

    pub fn pretty(&self, location: &str) -> String {
        let message = format!("could not parse {} at {location}", self.name);

        let snippet = Level::ERROR.primary_title(message).element({
            Snippet::source(self.input)
                .annotation(
                    AnnotationKind::Primary.span(self.location.clone().unwrap_or(0..0))
                        .label(self.error.to_string())
                )
        });

        Renderer::styled().anonymized_line_numbers(true).render(&[snippet]).to_string()
    }
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref location) = self.location {
            write!(f, "{} at `{}`", self.error, &self.input[location.clone()])
        } else {
            write!(f, "{} at `{}`", self.error, self.input)
        }
    }
}

impl fmt::Debug for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseError")
            .field("error", &self.error.to_string())
            .field("input", &self.input)
            .field("name", &self.name)
            .field("location", &self.location)
            .finish()
    }
}

impl<T: IntoGp + ?Sized> IntoGp for &T {
    fn into(&self) -> String {
        (*self).into()
    }
}

#[cfg(feature = "num")]
mod implement_rational {
    use std::fmt::Display;
    use num_bigint::BigInt;
    use num_rational::Ratio;
    use num_integer::Integer;
    use super::*;

    impl FromGp for BigInt {
        fn try_from<'s>(input: &'s str) -> Result<(Self, &'s str), ParseError<'s>> {
            let remainder = input.trim_start_matches(['+', '-'])
                .trim_start_matches(char::is_numeric);

            let input = &input[0..(input.len() - remainder.len())];

            let value = input.parse::<BigInt>()
                .map_err(
                    |err| ParseError::without_location(err.to_string(), input, Self::name())
                )?;
            
            Ok((value, remainder))
        }
    
        fn name() -> &'static str {
            "BigInt"
        }
    }

    impl IntoGp for BigInt {
        fn into(&self) -> String {
            self.to_string()
        }
    }

    impl<T> FromGp for Ratio<T>
        where T: FromGp + Clone + Integer
    {
        fn try_from<'s>(input: &'s str) -> Result<(Self, &'s str), ParseError<'s>> {
            let (num, remainder): (T, &_) = FromGp::try_from(input)?;
            let (den, remainder) = if let Some(rem) = remainder.trim_start().strip_prefix('/') {
                let (den, new_rem): (T, &_) = FromGp::try_from(rem.trim_start())?;

                if den.is_zero() {
                    return Err(ParseError::without_location(
                        "rational has zero denominator", input, Self::name()
                    ));
                } else {
                    (den, new_rem)
                }
            } else {
                return Ok((Ratio::from_integer(num), remainder))
            };

            Ok((Ratio::new(num, den), remainder))
        }

        fn name() -> &'static str {
            "Rational"
        }
    }

    impl<T> IntoGp for Ratio<T>
        where T: IntoGp + Display + Clone + Integer
    {
        fn into(&self) -> String {
            self.to_string()
        }
    }
}

mod implement_primitives {
    use nom::{
        error::{Error, ErrorKind, ParseError as NomParseError},
        Parser, IResult, Err,
        multi::separated_list0,
        sequence::{delimited, separated_pair},
        character::complete::{char, multispace0, none_of},
        bytes::complete::escaped,
    };
    use super::*;

    impl<'s> NomParseError<&'s str> for ParseError<'s> {
        fn from_error_kind(input: &'s str, kind: ErrorKind) -> Self {
            ParseError::without_location(
                kind.description().to_string(), input, "Unknown"
            )
        }

        fn append(_: &'s str, _: ErrorKind, other: Self) -> Self {
            other
        }
    } 

    fn parser<'s, T: FromGp>(input: &'s str) -> IResult<&'s str, T, ParseError<'s>> {
        FromGp::try_from(input)
            .map(|(ret, remainder)| (remainder, ret))
            .map_err(Err::Error)
    }

    fn parse_err<'s>(err: Err<ParseError<'s>>, name: &'static str) -> ParseError<'s> {
        match err {
            Err::Error(mut err) | Err::Failure(mut err) => {
                err.name = name;
                err
            },
            _ => unreachable!(),
        }
    }

    impl FromGp for String {
        fn try_from<'s>(input: &'s str) -> Result<(Self, &'s str), ParseError<'s>> {
            delimited(
                char('"'),
                escaped(none_of("\\\""), '\\', char('"')),
                char('"'),
            ).parse(input)
                .map(|(i, o)| (o.to_string(), i))
                .map_err(|err| parse_err(err, Self::name()))
        }
        
        fn name() -> &'static str {
            "String"
        }
    }

    impl IntoGp for String {
        fn into(&self) -> String {
            format!("\"{self}\"")
        }
    }

    impl<T: FromGp> FromGp for Vec<T> {
        fn try_from<'s>(input: &'s str) -> Result<(Self, &'s str), ParseError<'s>> {
            delimited(
                char('['),
                separated_list0(
                    char(','),
                    delimited(
                        multispace0,
                        parser::<T>,
                        multispace0,
                    )
                ),
                char(']')
            ).parse(input).map(|(i, o)| (o, i)).map_err(|err| parse_err(err, Self::name()))
        }

        fn name() -> &'static str {
            "Vec<_>"
        }
    }

    impl<T: IntoGp> IntoGp for Vec<T> {
        fn into(&self) -> String {
            IntoGp::into(self.as_slice())
        }
    }

    impl<T: IntoGp> IntoGp for [T] {
        fn into(&self) -> String {
            format!("[{}]", self.iter().map(|item| item.into()).collect::<Vec<_>>().join(", "))
        }
    }

    impl<T, U> FromGp for (T, U)
        where T: FromGp, U: FromGp
    {
        fn try_from<'s>(input: &'s str) -> Result<(Self, &'s str), ParseError<'s>> {
            delimited(
                char('['),
                separated_pair(
                    delimited(multispace0, parser::<T>, multispace0),
                    char(','),
                    delimited(multispace0, parser::<U>, multispace0),
                ),
                char(']')
            ).parse(input).map(|(i, o)| (o, i)).map_err(|err| parse_err(err, Self::name()))
        }

        fn name() -> &'static str {
            "(_, _)"
        }
    }

    impl<T, U> IntoGp for (T, U)
        where T: IntoGp, U: IntoGp
    {
        fn into(&self) -> String {
            format!(
                "[{}, {}]",
                IntoGp::into(&self.0), IntoGp::into(&self.1)
            )
        }
    }

    impl<T, U, V> IntoGp for (T, U, V)
        where T: IntoGp, U: IntoGp, V: IntoGp
    {
        fn into(&self) -> String {
            format!(
                "[{}, {}, {}]", 
                IntoGp::into(&self.0), IntoGp::into(&self.1), IntoGp::into(&self.2)
            )
        }
    }

    impl<T, U, V> FromGp for (T, U, V)
        where T: FromGp, U: FromGp, V: FromGp
    {
        fn try_from<'s>(input: &'s str) -> Result<(Self, &'s str), ParseError<'s>> {
            delimited(
                char('['),
                separated_pair(
                    delimited(multispace0, parser::<T>, multispace0),
                    char(','),
                    separated_pair(
                        delimited(multispace0, parser::<U>, multispace0),
                        char(','),
                        delimited(multispace0, parser::<V>, multispace0),
                    ),
                ),
                char(']')
            ).parse(input)
                .map(|(i, (t, (u, v)))| ((t, u, v), i))
                .map_err(|err| parse_err(err, Self::name()))
        }

        fn name() -> &'static str {
            "(_, _, _)"
        }
    }

    macro_rules! impl_primitive {
        ($($primitive:ident)+) => {
            $(
        impl FromGp for $primitive {
            fn try_from<'s>(input: &'s str) -> Result<(Self, &'s str), ParseError<'s>> {
                let (remainder, value) = nom::character::complete::$primitive(input)
                    .map_err(
                        |err: nom::Err<Error<&'s str>>|
                            ParseError::without_location(err.to_string(), input, Self::name())
                    )?;
                
                Ok((value, remainder))
            }

            fn name() -> &'static str {
                stringify!($primitive)
            }
        }

        impl IntoGp for $primitive {
            fn into(&self) -> String {
                self.to_string()
            }
        }
            )+
        }
    }

    impl_primitive!{ i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 usize isize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use num_rational::Ratio;

    fn verify<T: FromGp + IntoGp>(s: &str) {
        let (parsed, _) = match T::try_from(s) {
            Ok(n) => n,
            Err(_) => panic!("failed to parse `{s}` into {}", T::name())
        };

        let as_str = <T as IntoGp>::into(&parsed);

        assert_eq!(s, as_str);
    }

    #[test]
    fn parse_num() {
        let s = "9253845923452346502602680120245100230505000005606";

        for i in 2..s.len() {
            let input = &s[0..i];
            let minus = format!("-{input}");
            let minput = &minus;

            if i < 2 { verify::<u8>(input);    verify::<i8>(input);   verify::<i8>(minput);   }
            if i < 4 { verify::<u16>(input);   verify::<i16>(input);  verify::<i16>(minput);  }
            if i < 9 { verify::<u32>(input);   verify::<i32>(input);  verify::<i32>(minput);  }
            if i < 18 { verify::<u64>(input);  verify::<i64>(input);  verify::<i64>(minput);  }
            if i < 38 { verify::<u128>(input); verify::<i128>(input); verify::<i128>(minput); }

            verify::<BigInt>(input); verify::<BigInt>(minput);
            verify::<Ratio<BigInt>>(input); verify::<Ratio<BigInt>>(minput);
        }
    }

    #[test]
    fn parse_str() {
        verify::<String>("\"test\"");
    }
}