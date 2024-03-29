use std::iter::Inspect;

use super::tuple::tuple;
use crate::error::*;
use crate::span::Item;
use crate::traits::*;

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
            let r = p.parse(i.clone());

            match r {
                Ok((rest, matched)) => {
                    i = rest;
                    out.push(matched)
                }

                Err(e) => {
                    if e.is_fatal() {
                        return Err(e);
                    } else {
                        break;
                    }
                }
            }
        }

        Ok((i, out))
    }
}

pub fn many_until<I, O, PREDO, E, P, PRED>(
    mut p: P,
    mut pred: PRED,
) -> impl FnMut(I) -> Result<(I, Vec<O>), E>
where
    P: Parser<I, O, E>,
    PRED: Parser<I, PREDO, E>,
    I: Collection + Clone + Copy,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut out = vec![];

        loop {
            if i.length() == 0 {
                return Err(E::from_error_kind(
                    i,
                    ParseErrorKind::UntilNotMatched,
                    Severity::Error,
                ));
            }

            // Have we hit the predicate?
            let r = pred.parse(i.clone());

            match r {
                Ok((rest, _)) => return Ok((i, out)),
                Err(e) => {
                    if e.is_fatal() {
                        return Err(e)
                    }
                }
                _ => (),
            }

            let (rest, matched) = p.parse(i.clone())?;
            i = rest;
            out.push(matched)
        }

        Ok((i, out))
    }
}

pub fn many1<I, O, E, P>(mut p: P) -> impl FnMut(I) -> Result<(I, Vec<O>), E>
where
    P: Parser<I, O, E> + Clone,
    I: Clone,
    E: ParseError<I>,
{
    move |mut i: I| {
        let (rest, x) = p.clone().parse(i.clone())?;
        let mut out = vec![x];
        let (rest, xs) = many0(p.clone())(rest)?;
        out.extend(xs);
        Ok((i, out))
    }
}

pub fn preceded<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, O2), E>
where
    I: Clone,
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
pub fn succeeded<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, O1), E>
where
    I: Clone,
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |rest: I| {
        let (rest, matched_1) = first.parse(rest)?;
        let (rest, _) = second.parse(rest)?;
        Ok((rest, matched_1))
    }
}

pub fn pair<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, (O1, O2)), E>
where
    I: Clone,
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

pub fn opt<I, O, E, P>(mut first: P) -> impl FnMut(I) -> Result<(I, Option<O>), E>
where
    P: Parser<I, O, E>,
    E: ParseError<I>,
    I: Clone,
{
    move |input: I| {
        let ret = first.parse(input.clone());
        if ret.is_ok() {
            ret.map(|(r, m)| (r, Some(m)))
        } else {
            Ok((input, None))
        }
    }
}

pub fn not<I, O, E, P>(mut first: P) -> impl FnMut(I) -> Result<(I, I), E>
where
    I: Clone,
    P: Parser<I, O, E>,
    E: ParseError<I> + std::fmt::Debug,
{
    move |input: I| {
        let ret = first.parse(input.clone());

        match ret {
            Ok(r) => Err(E::from_error_kind(
                input.clone(),
                ParseErrorKind::NoMatch,
                Severity::Error,
            )),
            Err(mut e) => Ok((input.clone(), input.clone())),
        }
    }
}
pub fn all<I, O, E, P>(mut first: P) -> impl FnMut(I) -> Result<(I, O), E>
where
    I: Clone + Collection + Clone,
    P: Parser<I, O, E>,
    E: ParseError<I> + std::fmt::Debug,
{
    move |input: I| {
        let ret = first.parse(input.clone());
        match ret {
            Ok((rest, matched)) => {
                if rest.length() > 0 {
                    Err(E::from_error_kind(
                        input.clone(),
                        ParseErrorKind::UnconsumedInput,
                        Severity::Error,
                    ))
                } else {
                    Ok((rest, matched))
                }
            }
            Err(mut e) => Err(e.set_severity(Severity::Fatal)),
        }
    }
}

pub fn cut<I, O, E, P>(mut first: P) -> impl FnMut(I) -> Result<(I, O), E>
where
    I: Clone,
    P: Parser<I, O, E>,
    E: ParseError<I> + std::fmt::Debug,
{
    move |input: I| {
        let ret = first.parse(input);
        match ret {
            Ok(r) => Ok(r),
            Err(mut e) => Err(e.set_severity(Severity::Fatal)),
        }
    }
}

pub fn sep_pair<I, O1, O2, OS, P1, P2, PS, E>(
    mut first: P1,
    mut sep: PS,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, (O1, O2)), E>
where
    I: Clone,
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

pub fn wrapped<SP, OTHER, E, P, O>(
    open: OTHER,
    mut p: P,
    close: OTHER,
) -> impl FnMut(SP) -> Result<(SP, O), E>
where
    SP: Collection + Splitter<E> + Clone,
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
        let ret = rest.tag(close)?;
        Ok((rest, matched))
    }
}

pub fn wrapped_cut<SP, OTHER, E, P, O>(
    open: OTHER,
    mut p: P,
    close: OTHER,
) -> impl FnMut(SP) -> Result<(SP, O), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<OTHER as Collection>::Item as Item>::Kind>,

    OTHER: Collection + Copy,
    <OTHER as Collection>::Item: Item + Copy,

    E: ParseError<SP> + std::fmt::Debug,
    P: Parser<SP, O, E>,
{
    move |rest: SP| {
        let (rest, _) = rest.tag(open)?;
        let (rest, matched) = p.parse(rest)?;
        let (rest, _) = cut(tag(close))(rest)
            .map_err(|e| e.change_kind(ParseErrorKind::MissingWrapTerminator))?;
        Ok((rest, matched))
    }
}

pub fn tag<SP, OTHER, E>(tag: OTHER) -> impl FnMut(SP) -> Result<(SP, SP), E> + Clone
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

pub fn until<SP, E>(
    pred: impl Fn(<SP as Collection>::Item) -> bool,
) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: PartialEq + Item,
    E: ParseError<SP>,
{
    let r = move |input: SP| -> Result<(SP, SP), E> {
        if input.length() == 0 {
            Ok((input.clone(), input.clone()))
        } else {
            for index in 0..input.length() {
                let k = input.at(index).unwrap();

                if pred(k.clone()) {
                    return input.split_at(index);
                }
            }

            Err(ParseError::from_error(
                input,
                ParseErrorKind::UntilNotMatched,
            ))
        }
    };

    r
}

pub fn is_a<SP, C, E>(
    isa: C,
) -> impl FnMut(SP) -> Result<(SP, <<SP as Collection>::Item as Item>::Kind), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: PartialEq + Item,
    C: Collection,
    <C as Collection>::Item: PartialEq + Copy + Item,
    <<SP as Collection>::Item as Item>::Kind: PartialEq<<<C as Collection>::Item as Item>::Kind>,
    E: ParseError<SP>,
{
    let r = move |input: SP| -> Result<(SP, <<SP as Collection>::Item as Item>::Kind), E> {
        if input.length() == 0 {
            Err(ParseError::from_error(input, ParseErrorKind::NoMatch))
        } else {
            let k = input.at(0).map(|x| x.get_kind()).clone();

            for i in 0..isa.length() {
                let ik = isa.at(i).map(|x| x.get_kind());

                match (k.clone(), ik) {
                    (Some(a), Some(b)) => {
                        if a == b {
                            let r = input.drop(1).map(|x| (x, a.clone())).map_err(|_| {
                                ParseError::from_error(input, ParseErrorKind::NoMatch)
                            });
                            return r;
                        }
                    }
                    _ => panic!(),
                }
            }

            Err(ParseError::from_error(input, ParseErrorKind::NoMatch))
        }
    };

    r
}
