use crate::error::*;
use crate::traits::*;
use crate::span::Item;

// use thin_vec::{thin_vec, ThinVec};

pub fn many0<I, O, E, P>(mut p: P) -> impl FnMut(I) -> Result<(I, Vec<O>), E>
where
    P: Parser<I, O, E>,
    I: Clone,
    E: ParseError<I>,
{
    move |input: I| {
        let mut out = vec![];
        let mut i = input.clone();
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

trait Seq {}

pub fn pair<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, (O1, O2)), E>
where
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |input: I| {
        let (rest, matched_1) = first.parse(input)?;
        let (rest, matched_2) = second.parse(rest)?;
        Ok((rest, (matched_1, matched_2)))
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
    SP: Collection,
    <SP as Collection>::Item : PartialEq + Copy + Item,
{
    let r = move |input : SP| {
        panic!()
    };

    r
}





