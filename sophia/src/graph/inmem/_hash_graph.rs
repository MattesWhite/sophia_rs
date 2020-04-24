// this module is transparently re-exported by its parent `graph::inmem`

use std::collections::HashSet;
use std::convert::Infallible;
use std::hash::Hash;

use crate::graph::indexed::IndexedGraph;
use crate::graph::*;
use crate::triple::stream::{StreamResult, TripleSource};
use crate::triple::streaming_mode::{ByTermRefs, StreamedTriple};
use sophia_term::factory::TermFactory;
use sophia_term::index_map::TermIndexMap;
use sophia_term::{Term, TermData};

/// A generic implementation of [`Graph`] and [`MutableGraph`],
/// storing its terms in a [`TermIndexMap`],
/// and its triples in a [`HashSet`].
///
/// [`Graph`]: ../trait.Graph.html
/// [`MutableGraph`]: ../trait.MutableGraph.html
/// [`TermIndexMap`]: ../../term/index_map/trait.TermIndexMap.html
/// [`HashSet`]: https://doc.rust-lang.org/std/collections/struct.HashSet.html
#[derive(Default)]
pub struct HashGraph<I>
where
    I: TermIndexMap,
    I::Index: Hash,
    <I::Factory as TermFactory>::TermData: 'static,
{
    terms: I,
    triples: HashSet<[I::Index; 3]>,
}

impl<I> HashGraph<I>
where
    I: TermIndexMap,
    I::Index: Hash,
{
    pub fn new() -> HashGraph<I> {
        HashGraph {
            terms: I::default(),
            triples: HashSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.triples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }
}

impl<I> IndexedGraph for HashGraph<I>
where
    I: TermIndexMap,
    I::Index: Hash,
    <I::Factory as TermFactory>::TermData: 'static,
{
    type Index = I::Index;
    type TermData = <I::Factory as TermFactory>::TermData;

    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        HashGraph {
            terms: I::default(),
            triples: HashSet::with_capacity(capacity),
        }
    }

    #[inline]
    fn shrink_to_fit(&mut self) {
        self.terms.shrink_to_fit();
        self.triples.shrink_to_fit();
    }

    #[inline]
    fn get_index<T>(&self, t: &Term<T>) -> Option<Self::Index>
    where
        T: TermData,
    {
        self.terms.get_index(&t.as_ref_str())
    }

    #[inline]
    fn get_term(&'_ self, i: Self::Index) -> Option<&Term<Self::TermData>> {
        self.terms.get_term(i)
    }

    fn insert_indexed<T, U, V>(
        &mut self,
        s: &Term<T>,
        p: &Term<U>,
        o: &Term<V>,
    ) -> Option<[I::Index; 3]>
    where
        T: TermData,
        U: TermData,
        V: TermData,
    {
        let si = self.terms.make_index(&s.as_ref_str());
        let pi = self.terms.make_index(&p.as_ref_str());
        let oi = self.terms.make_index(&o.as_ref_str());
        let modified = self.triples.insert([si, pi, oi]);
        if modified {
            Some([si, pi, oi])
        } else {
            self.terms.dec_ref(si);
            self.terms.dec_ref(pi);
            self.terms.dec_ref(oi);
            None
        }
    }

    fn remove_indexed<T, U, V>(
        &mut self,
        s: &Term<T>,
        p: &Term<U>,
        o: &Term<V>,
    ) -> Option<[I::Index; 3]>
    where
        T: TermData,
        U: TermData,
        V: TermData,
    {
        let si = self.terms.get_index(&s.as_ref_str());
        let pi = self.terms.get_index(&p.as_ref_str());
        let oi = self.terms.get_index(&o.as_ref_str());
        if let (Some(si), Some(pi), Some(oi)) = (si, pi, oi) {
            let modified = self.triples.remove(&[si, pi, oi]);
            if modified {
                self.terms.dec_ref(si);
                self.terms.dec_ref(pi);
                self.terms.dec_ref(oi);
                return Some([si, pi, oi]);
            }
        }
        None
    }
}

impl<I> Graph for HashGraph<I>
where
    I: TermIndexMap,
    I::Index: Hash,
    <I::Factory as TermFactory>::TermData: 'static,
{
    type Triple = ByTermRefs<<Self as IndexedGraph>::TermData>;
    type Error = Infallible;

    fn triples(&self) -> GTripleSource<Self> {
        Box::from(self.triples.iter().map(move |[si, pi, oi]| {
            Ok(StreamedTriple::by_term_refs(
                self.terms.get_term(*si).unwrap(),
                self.terms.get_term(*pi).unwrap(),
                self.terms.get_term(*oi).unwrap(),
            ))
        }))
    }
}

impl<TS, I> CollectibleGraph<TS> for HashGraph<I>
where
    TS: TripleSource,
    I: TermIndexMap,
    I::Index: Hash,
    <I::Factory as TermFactory>::TermData: 'static,
{
    fn from_triple_source(triples: TS) -> StreamResult<Self, TS::Error, Infallible> {
        let (tmin, tmax) = triples.size_hint_triples();
        let cap = tmax.unwrap_or(tmin);
        let mut hash_graph = HashGraph {
            terms: I::default(),
            triples: if cap > 0 || tmax == Some(0) {
                HashSet::with_capacity(cap)
            } else {
                HashSet::default()
            },
        };
        hash_graph.insert_all(triples).map(|_| hash_graph)
    }
}

impl<I> MutableGraph for HashGraph<I>
where
    I: TermIndexMap,
    I::Index: Hash,
    <I::Factory as TermFactory>::TermData: 'static,
{
    impl_mutable_graph_for_indexed_graph!();
}

impl<I> SetGraph for HashGraph<I>
where
    I: TermIndexMap,
    I::Index: Hash,
{
}

#[cfg(test)]
mod test {
    // The code from this module is tested through its use in other modules
    // (especially in ./inmem.rs).
}
