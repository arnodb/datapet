use fallible_iterator::FallibleIterator;
use std::cmp::Ordering;

/// Sorts items in memory and stream them.
#[derive(new)]
pub struct Sort<I: FallibleIterator<Item = R, Error = E>, R, E, C>
where
    C: Fn(&R, &R) -> Ordering,
{
    input: I,
    cmp: C,
    #[new(default)]
    buffer: Vec<R>,
    #[new(value = "false")]
    finalizing: bool,
}

impl<I: FallibleIterator<Item = R, Error = E>, R, E, C> FallibleIterator for Sort<I, R, E, C>
where
    C: Fn(&R, &R) -> Ordering,
{
    type Item = R;
    type Error = E;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if !self.finalizing {
            while let Some(rec) = self.input.next()? {
                self.buffer.push(rec);
            }
            self.buffer.sort_by(|r1, r2| (self.cmp)(r1, r2).reverse());
            self.finalizing = true;
        }
        Ok(self.buffer.pop())
    }
}

#[test]
fn should_sort_stream() {
    let mut stream = Sort::new(
        fallible_iterator::convert(["ZZZ", "", "a", "z", "A"].into_iter().map(Ok::<_, ()>)),
        |a, b| {
            a.to_lowercase()
                .cmp(&b.to_lowercase())
                .then_with(|| a.cmp(b))
        },
    );
    assert_matches!(stream.next(), Ok(Some(v)) if v == "");
    assert_matches!(stream.next(), Ok(Some(v)) if v == "A");
    assert_matches!(stream.next(), Ok(Some(v)) if v == "a");
    assert_matches!(stream.next(), Ok(Some(v)) if v == "z");
    assert_matches!(stream.next(), Ok(Some(v)) if v == "ZZZ");
    // End of stream
    assert_matches!(stream.next(), Ok(None));
    assert_matches!(stream.next(), Ok(None));
}
