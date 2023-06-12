use super::prelude::*;

#[derive(Clone, Debug)]
pub struct Lines {
    offsets: Vec<std::ops::Range<usize>>,
}

fn compare_loc(offset: usize, x: &std::ops::Range<usize>) -> std::cmp::Ordering {
    if x.contains(&offset) {
        std::cmp::Ordering::Equal
    } else {
        if offset < x.start {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    }
}

impl Lines {
    pub fn new(text: &str) -> Self {
        let is_cr = |v| (v == b'\n');
        let filter = |(i, v)| is_cr(v).then_some(i);

        let offsets: Vec<_> = text.bytes().enumerate().filter_map(filter).collect();
        let x = vec![vec![0], offsets, vec![text.len()]]
            .into_iter()
            .flatten();
        let offsets = x.clone().zip(x.skip(1)).map(|(s, e)| s..e).collect();

        Self { offsets }
    }

    pub fn get_location_from_offset(&self, offset: usize) -> Option<Location> {
        let line = self.offsets.binary_search_by(|x| compare_loc(offset, x));

        match line {
            Ok(line) => {
                let line_start = self.offsets[line].start;
                let col = offset - line_start;

                Some(Location { line, col })
            }
            Err(_) => None,
        }
    }

    pub fn get_line_range(&self, line: usize) -> Option<std::ops::Range<usize>> {
        self.offsets.get(line).cloned()
    }
}
