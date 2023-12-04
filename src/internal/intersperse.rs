//! A nightly-only standard library method.
//! Once it is stablised we should remove this!
//! https://github.com/rust-lang/rust/issues/79524

use std::iter::Peekable;

#[derive(Debug, Clone)]
pub struct Intersperse<I: Iterator>
where
    I::Item: Clone,
{
    separator: I::Item,
    iter: Peekable<I>,
    needs_sep: bool,
}

impl<I: Iterator> Intersperse<I>
where
    I::Item: Clone,
{
    pub(crate) fn new(iter: I, separator: I::Item) -> Self {
        Self {
            iter: iter.peekable(),
            separator,
            needs_sep: false,
        }
    }
}

pub trait IntersperseIter: Iterator {
    fn _intersperse(self, separator: Self::Item) -> Intersperse<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        Intersperse::new(self, separator)
    }
}

impl<I: Iterator> IntersperseIter for I {}

impl<I> Iterator for Intersperse<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.needs_sep && self.iter.peek().is_some() {
            self.needs_sep = false;
            Some(self.separator.clone())
        } else {
            self.needs_sep = true;
            self.iter.next()
        }
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let separator = self.separator;
        intersperse_fold(
            self.iter,
            init,
            f,
            move || separator.clone(),
            self.needs_sep,
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        intersperse_size_hint(&self.iter, self.needs_sep)
    }
}

fn intersperse_size_hint<I>(iter: &I, needs_sep: bool) -> (usize, Option<usize>)
where
    I: Iterator,
{
    let (lo, hi) = iter.size_hint();
    let next_is_elem = !needs_sep;
    (
        lo.saturating_sub(next_is_elem as usize).saturating_add(lo),
        hi.and_then(|hi| hi.saturating_sub(next_is_elem as usize).checked_add(hi)),
    )
}

fn intersperse_fold<I, B, F, G>(
    mut iter: I,
    init: B,
    mut f: F,
    mut separator: G,
    needs_sep: bool,
) -> B
where
    I: Iterator,
    F: FnMut(B, I::Item) -> B,
    G: FnMut() -> I::Item,
{
    let mut accum = init;

    if !needs_sep {
        if let Some(x) = iter.next() {
            accum = f(accum, x);
        } else {
            return accum;
        }
    }

    iter.fold(accum, |mut accum, x| {
        accum = f(accum, separator());
        accum = f(accum, x);
        accum
    })
}
