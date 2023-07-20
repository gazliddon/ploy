pub type Span<'a> = unraveler::Span<'a, super::ploytokens::Token<'a>>;

pub fn get_text_range(input: Span) -> std::ops::Range<usize> {
    if input.is_empty() {
        let r = input.get_range();
        let start = &input.get_item_at_abs_position_sat(r.start).unwrap().extra;
        let start_t = start.as_range().start;
        if start_t + 1 < start.base.len() {
            start_t + 1..start_t + 1
        } else {
            start_t..start_t
        }
    } else {
        let start = input.as_slice().first().unwrap().extra.as_range();
        let end = input.as_slice().last().unwrap().extra.as_range();
        let start_t = start.start;
        let end_t = end.end;
        start_t..end_t
    }
}


