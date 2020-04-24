//! A quad source produces quads, and may also fail in the process.
//!
//! The trait [`QuadSource`]
//! provides an API similar to (a subset of) the [`Iterator`] API,
//! with methods such as [`for_each_quad`] and [`try_for_each_quad`].
//!
//! # Rationale (or Why not simply use `Iterator`?)
//!
//! See the documentation of the [`triple::stream`] module.
//!
//! [`QuadSource`]: trait.QuadSource.html
//! [`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
//! [`for_each_quad`]: ./trait.QuadSource.html#method.for_each_quad
//! [`try_for_each_quad`]: ./trait.QuadSource.html#method.try_for_each_quad
//! [`triple::stream`]: ../../triple/stream/index.html

use std::error::Error;

use crate::dataset::*;
use crate::quad::streaming_mode::*;
use crate::quad::*;
pub use crate::triple::stream::{SinkError, SourceError, StreamError, StreamResult};

mod _filter;
pub use _filter::*;
mod _filter_map;
pub use _filter_map::*;
mod _iterator;
pub use _iterator::*;
mod _map;
pub use _map::*;

/// Type alias for referencing the `TermData` used in a `QuadSource`.
pub type QSData<S> =
    <<<S as QuadSource>::Quad as QuadStreamingMode>::UnsafeQuad as UnsafeQuad>::TermData;

/// A quad source produces [quads], and may also fail in the process.
///
/// Any iterator yielding [quads] wrapped in `Result`
/// implements the `QuadSource` trait.
///
/// [quads]: ../trait.Quad.html
pub trait QuadSource {
    /// The type of errors produced by this source.
    type Error: 'static + Error;

    /// Determine the type of [`Quad`](../quad/trait.Quad.html)s
    /// that this quad source yields.
    /// (see [`streaming_mode`](../quad/streaming_mode/index.html)
    type Quad: QuadStreamingMode;

    /// Call f for at least one quad from this quad source, if any.
    ///
    /// Return false if there are no more quads in this source.
    fn try_for_some_quad<F, E>(&mut self, f: &mut F) -> StreamResult<bool, Self::Error, E>
    where
        F: FnMut(StreamedQuad<Self::Quad>) -> Result<(), E>,
        E: Error;

    /// Call f for all quads from this quad source.
    #[inline]
    fn try_for_each_quad<F, E>(&mut self, f: F) -> StreamResult<(), Self::Error, E>
    where
        F: FnMut(StreamedQuad<Self::Quad>) -> Result<(), E>,
        E: Error,
    {
        let mut f = f;
        while self.try_for_some_quad(&mut f)? {}
        Ok(())
    }
    /// Call f for at least one quad from this quad source, if any.
    ///
    /// Return false if there are no more quads in this source.
    #[inline]
    fn for_some_quad<F>(&mut self, f: &mut F) -> Result<bool, Self::Error>
    where
        F: FnMut(StreamedQuad<Self::Quad>) -> (),
    {
        self.try_for_some_quad(&mut |t| -> Result<(), Self::Error> {
            f(t);
            Ok(())
        })
        .map_err(StreamError::inner_into)
    }
    /// Call f for all quads from this quad source.
    #[inline]
    fn for_each_quad<F>(&mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnMut(StreamedQuad<Self::Quad>) -> (),
    {
        let mut f = f;
        while self.for_some_quad(&mut f)? {}
        Ok(())
    }
    /// Creates a quad source which uses a closure to determine if a quad should be yielded.
    #[inline]
    fn filter_quads<F>(self, filter: F) -> FilterSource<Self, F>
    where
        Self: Sized,
        F: FnMut(&StreamedQuad<Self::Quad>) -> bool,
    {
        FilterSource {
            source: self,
            filter,
        }
    }
    /// Creates a quad source that both filters and maps.
    #[inline]
    fn filter_map_quads<F, T>(self, filter_map: F) -> FilterMapSource<Self, F>
    where
        Self: Sized,
        F: FnMut(StreamedQuad<Self::Quad>) -> Option<T>,
    {
        FilterMapSource {
            source: self,
            filter_map,
        }
    }
    /// Takes a closure and creates quad source which yield the result of that closure for each quad.
    #[inline]
    fn map_quads<F, T>(self, map: F) -> MapSource<Self, F>
    where
        Self: Sized,
        F: FnMut(StreamedQuad<Self::Quad>) -> T,
    {
        MapSource { source: self, map }
    }
    /// Returns the bounds on the remaining length of the quad source.
    ///
    /// This method has the same contract as [`Iterator::size_hint`].
    ///
    /// [`Iterator::size_hint`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.size_hint
    fn size_hint_quads(&self) -> (usize, Option<usize>) {
        (0, None)
    }
    /// Collect these quads into a new dataset.
    fn collect_quads<D>(self) -> StreamResult<D, Self::Error, <D as Dataset>::Error>
    where
        Self: Sized,
        D: CollectibleDataset<Self>,
    {
        D::from_quad_source(self)
    }
    /// Insert all quads from this source into the given [dataset](../../dataset/trait.MutableDataset.html).
    ///
    /// Stop on the first error (in the source or in the dataset).
    #[inline]
    fn add_to_dataset<D: MutableDataset>(
        self,
        dataset: &mut D,
    ) -> StreamResult<usize, Self::Error, <D as MutableDataset>::MutationError>
    where
        Self: Sized,
    {
        dataset.insert_all(self)
    }
}

#[cfg(test)]
mod test;
