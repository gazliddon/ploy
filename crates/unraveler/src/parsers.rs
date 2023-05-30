use crate::error::*;
use crate::traits::*;
use crate::span::Item;
use super::tuple::tuple;

// use thin_vec::{thin_vec, ThinVec};

pub fn many0<I, O, E, P>(mut p: P) -> impl FnMut(I) -> Result<(I, Vec<O>), E>
where
    P: Parser<I, O, E>,
    I: Clone,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut out = vec![];

        loop {
            if let Ok((rest, matched)) = p.parse(i.clone()) {
                i = rest;
                out.push(matched)
            } else {
                break;
            }
        }

        Ok((i, out))
    }
}
pub fn many1<I, O, E, P>(mut p: P) -> impl FnMut(I) -> Result<(I, Vec<O>), E>
where
    P: Parser<I, O, E>,
    I: Clone,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut out = vec![];

        loop {
            if let Ok((rest, matched)) = p.parse(i.clone()) {
                i = rest;
                out.push(matched)
            } else {
                break;
            }
        }

        if out.is_empty() {
            Err(ParseError::from_error_kind(&i, ParseErrorKind::NeededOneOrMore))
        } else {
            Ok((i, out))
        }
    }
}

pub fn preceded<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, O2), E>
where
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |rest: I| {
        let (rest, _) = first.parse(rest)?;
        let (rest, matched_2) = second.parse(rest)?;
        Ok((rest, matched_2))
    }
}

pub fn pair<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, (O1, O2)), E>
where
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |rest: I| {
        let (rest, matched_1) = first.parse(rest)?;
        let (rest, matched_2) = second.parse(rest)?;
        Ok((rest, (matched_1, matched_2)))
    }
}

pub fn sep_pair<I, O1, O2, OS, P1, P2, PS, E>(
    mut first: P1,
    mut sep: PS,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, (O1, O2)), E>
where
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    PS: Parser<I, OS, E>,
    E: ParseError<I>,
{
    move |input: I| {
        let (rest, matched_1) = first.parse(input)?;
        let (rest, _) = sep.parse(rest)?;
        let (rest, matched_2) = second.parse(rest)?;
        Ok((rest, (matched_1, matched_2)))
    }
}

pub fn wrapped<SP, OTHER, E,P,O>(open: OTHER, mut p : P, close: OTHER) -> impl FnMut(SP) -> Result<(SP, O), E>
where
    SP: Collection + Splitter<E>,
    <SP as Collection>::Item: Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<OTHER as Collection>::Item as Item>::Kind>,

    OTHER: Collection + Copy,
    <OTHER as Collection>::Item: Item + Copy,

    E: ParseError<SP>,
    P: Parser<SP, O, E>,
{
    move |rest: SP| {
        let (rest, _) = rest.tag(open)?;
        let (rest, matched) = p.parse(rest)?;
        let (rest, _) = rest.tag(close)?;
        Ok((rest, matched))
    }
}

pub fn tag<SP, OTHER, E>(tag: OTHER) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E>,
    <SP as Collection>::Item: Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<OTHER as Collection>::Item as Item>::Kind>,

    OTHER: Collection + Copy,
    <OTHER as Collection>::Item: Item + Copy,
    E: ParseError<SP>,
{
    move |input: SP| {
        let (rest, matched) = input.tag(tag)?;
        Ok((rest, matched))
    }
}

pub fn any<SP, E>() -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Splitter<E>,
    E: ParseError<SP>,
{
    move |input: SP| input.split_at(1)
}

pub fn is_a<SP,C,E>(isa: C) -> impl FnMut(SP) -> Result<(SP, <<SP as Collection>::Item as Item>::Kind), E> 
   where 
    SP: Collection + Splitter<E>,
    <SP as Collection>::Item : PartialEq + Copy + Item,
    C: Collection,
    <C as Collection>::Item : PartialEq + Copy + Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<C as Collection>::Item as Item>::Kind>,
    E: ParseError<SP>,
{
    let r = move |input : SP| -> Result<( SP,<<SP as Collection>::Item as Item>::Kind ),E>{
        if input.length() == 0 {
            Err(ParseError::from_error_kind(&input, ParseErrorKind::NoMatch))
        } else {
            let k = input.at(0).map(|x| x.get_kind());

            for i in 0..isa.length() {
                let ik = isa.at(i).map(|x|x.get_kind());

               match (k,ik ) {
                    (Some(a), Some(b)) => if a==b {
                        let r = input.drop(1).map(|x| (x,a)).map_err(|_| ParseError::from_error_kind(&input, ParseErrorKind::NoMatch));
                        return r;

                    },
                    _ => panic!()
                }
            }

            Err(ParseError::from_error_kind(&input, ParseErrorKind::NoMatch))
        }
    };

    r
}





