// this module is transparently re-exported by its parent `dataset`

use std::collections::HashSet;
use std::convert::Infallible;
use std::error::Error;
use std::marker::PhantomData;

use resiter::filter::*;
use resiter::map::*;

use crate::dataset::adapter::DatasetGraph;
use crate::quad::stream::*;
use crate::quad::streaming_mode::*;
use crate::quad::*;
use crate::triple::stream::StreamResult;
use sophia_term::matcher::*;
use sophia_term::*;

use crate::graph::insert_if_absent;

/// Type alias for the terms returned by a dataset.
pub type DTerm<D> =
    Term<<<<D as Dataset>::Quad as QuadStreamingMode>::UnsafeQuad as UnsafeQuad>::TermData>;
/// Type alias for the quads returned by a dataset.
pub type DQuad<'a, D> = StreamedQuad<'a, <D as Dataset>::Quad>;
/// Type alias for results iterators produced by a dataset.
pub type DResult<D, T> = Result<T, <D as Dataset>::Error>;
/// Type alias for fallible quad iterators produced by a dataset.
pub type DQuadSource<'a, D> = Box<dyn Iterator<Item = DResult<D, DQuad<'a, D>>> + 'a>;
/// Type alias for fallible hashsets of terms produced by a dataset.
///
/// See [`Dataset::quads`](./trait.Dataset.html#tymethod.quads)
/// for more information about how to use it.
pub type DResultTermSet<D> = DResult<D, HashSet<DTerm<D>>>;

/// Generic trait for RDF datasets.
///
/// For convenience, this trait is implemented
/// by [standard collections of quads](#foreign-impls).
///
/// NB: the semantics of this trait allows a dataset to contain duplicate quads;
/// see also [`SetDataset`](trait.SetDataset.html).
pub trait Dataset {
    /// Determine the type of [`Quad`](../quad/trait.Quad.html)s
    /// that the methods of this dataset will yield
    /// (see [`streaming_mode`](../quad/streaming_mode/index.html)
    type Quad: QuadStreamingMode;
    /// The error type that this dataset may raise.
    type Error: 'static + Error;

    /// An iterator visiting all quads of this dataset in arbitrary order.
    ///
    /// This iterator is fallible:
    /// its items are `Result`s,
    /// an error may occur at any time during the iteration.
    ///
    /// # Examples
    ///
    /// The result of this method is an iterator,
    /// so it can be used in a `for` loop:
    /// ```
    /// # use sophia::dataset::Dataset;
    /// # use sophia::term::BoxTerm;
    /// # fn foo() -> Result<(), std::convert::Infallible> {
    /// # let dataset = Vec::<[BoxTerm;4]>::new();
    /// for q in dataset.quads() {
    ///     let q = q?; // rethrow error if any
    ///     // do something with q
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Another way is to use the specific methods provided by
    /// [`QuadSource`](../quad/stream/trait.QuadSource.html),
    /// for example:
    /// ```
    /// # use sophia::dataset::Dataset;
    /// # use sophia::term::BoxTerm;
    /// # use sophia::quad::stream::QuadSource;
    /// # fn foo() -> Result<(), std::convert::Infallible> {
    /// # let dataset = Vec::<[BoxTerm;4]>::new();
    /// dataset.quads().for_each_quad(|q| {
    ///     // do something with q
    /// })?; // rethrow error if any
    /// # Ok(())
    /// # }
    /// ```
    fn quads(&self) -> DQuadSource<Self>;

    /// An iterator visiting all quads with the given subject.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_s<'s, T>(&'s self, s: &'s Term<T>) -> DQuadSource<'s, Self>
    where
        T: TermData,
    {
        Box::new(self.quads().filter_ok(move |q| q.s() == s))
    }
    /// An iterator visiting all quads with the given predicate.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_p<'s, T>(&'s self, p: &'s Term<T>) -> DQuadSource<'s, Self>
    where
        T: TermData,
    {
        Box::new(self.quads().filter_ok(move |q| q.p() == p))
    }
    /// An iterator visiting add quads with the given object.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_o<'s, T>(&'s self, o: &'s Term<T>) -> DQuadSource<'s, Self>
    where
        T: TermData,
    {
        Box::new(self.quads().filter_ok(move |q| q.o() == o))
    }
    /// An iterator visiting add quads with the given graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_g<'s, T>(&'s self, g: Option<&'s Term<T>>) -> DQuadSource<'s, Self>
    where
        T: TermData,
    {
        Box::new(self.quads().filter_ok(move |q| same_graph_name(q.g(), g)))
    }
    /// An iterator visiting add quads with the given subject and predicate.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_sp<'s, T, U>(&'s self, s: &'s Term<T>, p: &'s Term<U>) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
    {
        Box::new(self.quads_with_s(s).filter_ok(move |q| q.p() == p))
    }
    /// An iterator visiting add quads with the given subject and object.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_so<'s, T, U>(&'s self, s: &'s Term<T>, o: &'s Term<U>) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
    {
        Box::new(self.quads_with_s(s).filter_ok(move |q| q.o() == o))
    }
    /// An iterator visiting add quads with the given subject and graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_sg<'s, T, U>(
        &'s self,
        s: &'s Term<T>,
        g: Option<&'s Term<U>>,
    ) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
    {
        Box::new(self.quads_with_g(g).filter_ok(move |q| q.s() == s))
    }
    /// An iterator visiting add quads with the given predicate and object.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_po<'s, T, U>(&'s self, p: &'s Term<T>, o: &'s Term<U>) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
    {
        Box::new(self.quads_with_p(p).filter_ok(move |q| q.o() == o))
    }
    /// An iterator visiting add quads with the given predicate and graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_pg<'s, T, U>(
        &'s self,
        p: &'s Term<T>,
        g: Option<&'s Term<U>>,
    ) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
    {
        Box::new(self.quads_with_g(g).filter_ok(move |q| q.p() == p))
    }
    /// An iterator visiting add quads with the given object and graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_og<'s, T, U>(
        &'s self,
        o: &'s Term<T>,
        g: Option<&'s Term<U>>,
    ) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
    {
        Box::new(self.quads_with_g(g).filter_ok(move |q| q.o() == o))
    }
    /// An iterator visiting add quads with the given subject, predicate and object.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_spo<'s, T, U, V>(
        &'s self,
        s: &'s Term<T>,
        p: &'s Term<U>,
        o: &'s Term<V>,
    ) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
        V: TermData,
    {
        Box::new(self.quads_with_sp(s, p).filter_ok(move |q| q.o() == o))
    }
    /// An iterator visiting add quads with the given subject, predicate and graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_spg<'s, T, U, V>(
        &'s self,
        s: &'s Term<T>,
        p: &'s Term<U>,
        g: Option<&'s Term<V>>,
    ) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
        V: TermData,
    {
        Box::new(self.quads_with_sg(s, g).filter_ok(move |q| q.p() == p))
    }
    /// An iterator visiting add quads with the given subject, object and graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_sog<'s, T, U, V>(
        &'s self,
        s: &'s Term<T>,
        o: &'s Term<U>,
        g: Option<&'s Term<V>>,
    ) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
        V: TermData,
    {
        Box::new(self.quads_with_sg(s, g).filter_ok(move |q| q.o() == o))
    }
    /// An iterator visiting add quads with the given predicate, object and graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_pog<'s, T, U, V>(
        &'s self,
        p: &'s Term<T>,
        o: &'s Term<U>,
        g: Option<&'s Term<V>>,
    ) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
        V: TermData,
    {
        Box::new(self.quads_with_pg(p, g).filter_ok(move |q| q.o() == o))
    }
    /// An iterator visiting add quads with the given subject, predicate, object and graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_with_spog<'s, T, U, V, W>(
        &'s self,
        s: &'s Term<T>,
        p: &'s Term<U>,
        o: &'s Term<V>,
        g: Option<&'s Term<W>>,
    ) -> DQuadSource<'s, Self>
    where
        T: TermData,
        U: TermData,
        V: TermData,
        W: TermData,
    {
        Box::new(self.quads_with_spg(s, p, g).filter_ok(move |q| q.o() == o))
    }

    /// Return `true` if this dataset contains the given quad.
    fn contains<T, U, V, W>(
        &self,
        s: &Term<T>,
        p: &Term<U>,
        o: &Term<V>,
        g: Option<&Term<W>>,
    ) -> DResult<Self, bool>
    where
        T: TermData,
        U: TermData,
        V: TermData,
        W: TermData,
    {
        match self.quads_with_spog(s, p, o, g).next() {
            None => Ok(false),
            Some(Ok(_)) => Ok(true),
            Some(Err(err)) => Err(err),
        }
    }

    /// An iterator visiting add quads matching the given subject, predicate, object and graph name.
    ///
    /// See also [`quads`](#tymethod.quads).
    fn quads_matching<'s, S, P, O, G>(
        &'s self,
        ms: &'s S,
        mp: &'s P,
        mo: &'s O,
        mg: &'s G,
    ) -> DQuadSource<'s, Self>
    where
        S: TermMatcher + ?Sized,
        P: TermMatcher + ?Sized,
        O: TermMatcher + ?Sized,
        G: GraphNameMatcher + ?Sized,
    {
        match (
            &ms.constant(),
            &mp.constant(),
            &mo.constant(),
            &mg.constant(),
        ) {
            (None, None, None, None) => Box::from(self.quads().filter_ok(move |q| {
                ms.matches(q.s()) && mp.matches(q.p()) && mo.matches(q.o()) && mg.matches(q.g())
            })),
            (Some(s), None, None, None) => {
                Box::from(self.quads_with_s(s).filter_ok(move |q| {
                    mp.matches(q.p()) && mo.matches(q.o()) && mg.matches(q.g())
                }))
            }
            (None, Some(p), None, None) => {
                Box::from(self.quads_with_p(p).filter_ok(move |q| {
                    ms.matches(q.s()) && mo.matches(q.o()) && mg.matches(q.g())
                }))
            }
            (None, None, Some(o), None) => {
                Box::from(self.quads_with_o(o).filter_ok(move |q| {
                    ms.matches(q.s()) && mp.matches(q.p()) && mg.matches(q.g())
                }))
            }
            (None, None, None, Some(g)) => {
                Box::from(self.quads_with_g(*g).filter_ok(move |q| {
                    ms.matches(q.s()) && mp.matches(q.p()) && mo.matches(q.o())
                }))
            }
            (Some(s), Some(p), None, None) => Box::from(
                self.quads_with_sp(s, p)
                    .filter_ok(move |q| mo.matches(q.o()) && mg.matches(q.g())),
            ),
            (Some(s), None, Some(o), None) => Box::from(
                self.quads_with_so(s, o)
                    .filter_ok(move |q| mp.matches(q.p()) && mg.matches(q.g())),
            ),
            (Some(s), None, None, Some(g)) => Box::from(
                self.quads_with_sg(s, *g)
                    .filter_ok(move |q| mp.matches(q.p()) && mo.matches(q.o())),
            ),
            (None, Some(p), Some(o), None) => Box::from(
                self.quads_with_po(p, o)
                    .filter_ok(move |q| ms.matches(q.s()) && mg.matches(q.g())),
            ),
            (None, Some(p), None, Some(g)) => Box::from(
                self.quads_with_pg(p, *g)
                    .filter_ok(move |q| ms.matches(q.s()) && mo.matches(q.o())),
            ),
            (None, None, Some(o), Some(g)) => Box::from(
                self.quads_with_og(o, *g)
                    .filter_ok(move |q| ms.matches(q.s()) && mp.matches(q.p())),
            ),
            (Some(s), Some(p), Some(o), None) => Box::from(
                self.quads_with_spo(s, p, o)
                    .filter_ok(move |q| mg.matches(q.g())),
            ),
            (Some(s), Some(p), None, Some(g)) => Box::from(
                self.quads_with_spg(s, p, *g)
                    .filter_ok(move |q| mo.matches(q.o())),
            ),
            (Some(s), None, Some(o), Some(g)) => Box::from(
                self.quads_with_sog(s, o, *g)
                    .filter_ok(move |q| mp.matches(q.p())),
            ),
            (None, Some(p), Some(o), Some(g)) => Box::from(
                self.quads_with_pog(p, o, *g)
                    .filter_ok(move |q| ms.matches(q.s())),
            ),
            (Some(s), Some(p), Some(o), Some(g)) => self.quads_with_spog(s, p, o, *g),
        }
    }

    /// Build a Hashset of all the terms used as subject in this Dataset.
    fn subjects(&self) -> DResultTermSet<Self> {
        let mut res = std::collections::HashSet::new();
        for q in self.quads() {
            insert_if_absent(&mut res, q?.s());
        }
        Ok(res)
    }

    /// Build a Hashset of all the terms used as predicate in this Dataset.
    fn predicates(&self) -> DResultTermSet<Self> {
        let mut res = std::collections::HashSet::new();
        for q in self.quads() {
            insert_if_absent(&mut res, q?.p());
        }
        Ok(res)
    }

    /// Build a Hashset of all the terms used as object in this Dataset.
    fn objects(&self) -> DResultTermSet<Self> {
        let mut res = std::collections::HashSet::new();
        for q in self.quads() {
            insert_if_absent(&mut res, q?.o());
        }
        Ok(res)
    }

    /// Build a Hashset of all the terms used as graph names in this Dataset.
    fn graph_names(&self) -> DResultTermSet<Self> {
        let mut res = std::collections::HashSet::new();
        for q in self.quads() {
            let q = q?;
            let name = q.g();
            if let Some(name) = name {
                insert_if_absent(&mut res, name);
            }
        }
        Ok(res)
    }

    /// Build a Hashset of all the IRIs used in this Dataset.
    fn iris(&self) -> DResultTermSet<Self> {
        let mut res = std::collections::HashSet::new();
        for q in self.quads() {
            let q = q?;
            let (s, p, o) = (q.s(), q.p(), q.o());
            if let Term::Iri(_) = s {
                insert_if_absent(&mut res, s)
            }
            if let Term::Iri(_) = p {
                insert_if_absent(&mut res, p)
            }
            if let Term::Iri(_) = o {
                insert_if_absent(&mut res, o)
            }
            if let Some(gn) = q.g() {
                if let Term::Iri(_) = gn {
                    insert_if_absent(&mut res, &gn)
                }
            }
        }
        Ok(res)
    }

    /// Build a Hashset of all the BNodes used in this Dataset.
    fn bnodes(&self) -> DResultTermSet<Self> {
        let mut res = std::collections::HashSet::new();
        for q in self.quads() {
            let q = q?;
            let (s, p, o) = (q.s(), q.p(), q.o());
            if let Term::BNode(_) = s {
                insert_if_absent(&mut res, s)
            }
            if let Term::BNode(_) = p {
                insert_if_absent(&mut res, p)
            }
            if let Term::BNode(_) = o {
                insert_if_absent(&mut res, o)
            }
            if let Some(gn) = q.g() {
                if let Term::BNode(_) = gn {
                    insert_if_absent(&mut res, &gn)
                }
            }
        }
        Ok(res)
    }

    /// Build a Hashset of all the Literals used in this Dataset.
    fn literals(&self) -> DResultTermSet<Self> {
        let mut res = std::collections::HashSet::new();
        for q in self.quads() {
            let q = q?;
            let (s, p, o) = (q.s(), q.p(), q.o());
            if let Term::Literal(_) = s {
                insert_if_absent(&mut res, s)
            }
            if let Term::Literal(_) = p {
                insert_if_absent(&mut res, p)
            }
            if let Term::Literal(_) = o {
                insert_if_absent(&mut res, o)
            }
            if let Some(gn) = q.g() {
                if let Term::Literal(_) = gn {
                    insert_if_absent(&mut res, &gn)
                }
            }
        }
        Ok(res)
    }

    /// Build a Hashset of all the variables used in this Dataset.
    fn variables(&self) -> DResultTermSet<Self> {
        let mut res = std::collections::HashSet::new();
        for q in self.quads() {
            let q = q?;
            let (s, p, o) = (q.s(), q.p(), q.o());
            if let Term::Variable(_) = s {
                insert_if_absent(&mut res, s)
            }
            if let Term::Variable(_) = p {
                insert_if_absent(&mut res, p)
            }
            if let Term::Variable(_) = o {
                insert_if_absent(&mut res, o)
            }
            if let Some(gn) = q.g() {
                if let Term::Variable(_) = gn {
                    insert_if_absent(&mut res, &gn)
                }
            }
        }
        Ok(res)
    }

    /// Borrows one of the graphs of this dataset
    fn graph<T>(&self, graph_name: Option<&Term<T>>) -> DatasetGraph<Self, &Self, Option<BoxTerm>>
    where
        T: TermData,
    {
        DatasetGraph {
            dataset: self,
            gmatcher: graph_name.map(|n| n.clone_into()),
            _phantom: PhantomData,
        }
    }

    /// Borrows mutably one of the graphs of this dataset
    fn graph_mut<T>(
        &mut self,
        graph_name: Option<&Term<T>>,
    ) -> DatasetGraph<Self, &mut Self, Option<BoxTerm>>
    where
        T: TermData,
    {
        DatasetGraph {
            dataset: self,
            gmatcher: graph_name.map(|n| n.clone_into()),
            _phantom: PhantomData,
        }
    }

    /// Borrows a graph containing the union of all graphs matched by `gmatcher`
    fn union_graph<T>(&self, gmatcher: T) -> DatasetGraph<Self, &Self, T>
    where
        T: GraphNameMatcher,
    {
        DatasetGraph {
            dataset: self,
            gmatcher,
            _phantom: PhantomData,
        }
    }
}

/// A dataset that can be constructed from a
/// [`QuadSource`](../quad/stream/trait.QuadSource.html)
pub trait CollectibleDataset<QS>: Dataset + Sized
where
    QS: QuadSource,
{
    fn from_quad_source(quad: QS) -> StreamResult<Self, QS::Error, Self::Error>;
}

/// Type alias for results produced by a mutable dataset.
pub type MDResult<D, T> = std::result::Result<T, <D as MutableDataset>::MutationError>;

/// Generic trait for mutable RDF datasets.
///
/// NB: the semantics of this trait allows a dataset to contain duplicate quads;
/// see also [`SetDataset`](trait.SetDataset.html).
///
pub trait MutableDataset: Dataset {
    /// The error type that this dataset may raise during mutations.
    type MutationError: 'static + Error;

    /// Insert the given quad in this dataset.
    ///
    /// # Return value
    /// The `bool` value returned in case of success is
    /// **not significant unless** this dataset also implements [`SetDataset`].
    ///
    /// If it does,
    /// `true` is returned iff the insertion actually changed the dataset.
    /// In other words,
    /// a return value of `false` means that the dataset was not changed,
    /// because the quad was already present in this [`SetDataset`].
    ///
    /// [`SetDataset`]: trait.SetDataset.html
    fn insert<T, U, V, W>(
        &mut self,
        s: &Term<T>,
        p: &Term<U>,
        o: &Term<V>,
        g: Option<&Term<W>>,
    ) -> MDResult<Self, bool>
    where
        T: TermData,
        U: TermData,
        V: TermData,
        W: TermData;

    /// Remove the given quad from this dataset.
    ///
    /// # Return value
    /// The `bool` value returned in case of success is
    /// **not significant unless** this dataset also implements [`SetDataset`].
    ///
    /// If it does,
    /// `true` is returned iff the removal actually changed the dataset.
    /// In other words,
    /// a return value of `false` means that the dataset was not changed,
    /// because the quad was already absent from this [`SetDataset`].
    ///
    /// [`SetDataset`]: trait.SetDataset.html
    fn remove<T, U, V, W>(
        &mut self,
        s: &Term<T>,
        p: &Term<U>,
        o: &Term<V>,
        g: Option<&Term<W>>,
    ) -> MDResult<Self, bool>
    where
        T: TermData,
        U: TermData,
        V: TermData,
        W: TermData;

    /// Insert into this dataset all quads from the given source.
    ///
    /// # Blank node scope
    /// The blank nodes contained in the quad source will be inserted as is.
    /// If they happen to have the same identifier as blank nodes already present,
    /// they will be considered equal.
    /// This might *not* be what you want,
    /// especially if the dataset contains data from a file,
    /// and you are inserting data from a different file.
    /// In that case, you should first transform the quad source,
    /// in order to get fresh blank node identifiers.
    ///
    /// # Return value
    /// The `usize` value returned in case of success is
    /// **not significant unless** this dataset also implements [`SetDataset`].
    ///
    /// If it does,
    /// the number of quads that were *actually* inserted
    /// (i.e. that were not already present in this [`SetDataset`])
    /// is returned.
    ///
    /// [`SetDataset`]: trait.SetDataset.html
    #[inline]
    fn insert_all<QS>(
        &mut self,
        src: QS,
    ) -> StreamResult<usize, QS::Error, <Self as MutableDataset>::MutationError>
    where
        QS: QuadSource,
    {
        let mut src = src;
        let mut c = 0;
        src.try_for_each_quad(|q| -> MDResult<Self, ()> {
            if self.insert(q.s(), q.p(), q.o(), q.g())? {
                c += 1;
            }
            Ok(())
        })
        .and(Ok(c))
    }

    /// Remove from this dataset all quads from the given source.
    ///
    /// # Return value
    /// The `usize` value returned in case of success is
    /// **not significant unless** this dataset also implements [`SetDataset`].
    ///
    /// If it does,
    /// the number of quads that were *actually* removed
    /// (i.e. that were not already absent from this [`SetDataset`])
    /// is returned.
    ///
    /// [`SetDataset`]: trait.SetDataset.html
    #[inline]
    fn remove_all<QS>(
        &mut self,
        src: QS,
    ) -> StreamResult<usize, QS::Error, <Self as MutableDataset>::MutationError>
    where
        QS: QuadSource,
    {
        let mut src = src;
        let mut c = 0;
        src.try_for_each_quad(|q| -> MDResult<Self, ()> {
            if self.remove(q.s(), q.p(), q.o(), q.g())? {
                c += 1;
            }
            Ok(())
        })
        .and(Ok(c))
    }

    /// Remove all quads matching the given matchers.
    ///
    /// # Return value
    /// The `usize` value returned in case of success is
    /// **not significant unless** this dataset also implements [`SetDataset`].
    ///
    /// If it does,
    /// the number of quads that were *actually* removed
    /// (i.e. that were not already absent from this [`SetDataset`])
    /// is returned.
    ///
    /// # Note to implementors
    /// The default implementation is rather naive,
    /// and could be improved in specific implementations of the trait.
    ///
    /// [`SetDataset`]: trait.SetDataset.html
    fn remove_matching<S, P, O, G>(
        &mut self,
        ms: &S,
        mp: &P,
        mo: &O,
        mg: &G,
    ) -> MDResult<Self, usize>
    where
        S: TermMatcher + ?Sized,
        P: TermMatcher + ?Sized,
        O: TermMatcher + ?Sized,
        G: GraphNameMatcher + ?Sized,
        <Self as Dataset>::Error: Into<Self::MutationError>,
        Infallible: Into<Self::MutationError>,
    {
        let to_remove = self
            .quads_matching(ms, mp, mo, mg)
            .map_ok(|q| {
                (
                    [
                        q.s().clone_into::<Box<str>>(),
                        q.p().clone_into::<Box<str>>(),
                        q.o().clone_into::<Box<str>>(),
                    ],
                    q.g().map(|g| g.clone_into::<Box<str>>()),
                )
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)?;
        let mut to_remove = to_remove.into_iter().as_quad_source();
        Ok(self
            .remove_all(&mut to_remove)
            .map_err(|err| err.inner_into())?)
    }

    /// Keep only the quads matching the given matchers.
    ///
    /// # Note to implementors
    /// The default implementation is rather naive,
    /// and could be improved in specific implementations of the trait.
    fn retain_matching<S, P, O, G>(&mut self, ms: &S, mp: &P, mo: &O, mg: &G) -> MDResult<Self, ()>
    where
        S: TermMatcher + ?Sized,
        P: TermMatcher + ?Sized,
        O: TermMatcher + ?Sized,
        G: GraphNameMatcher + ?Sized,
        <Self as Dataset>::Error: Into<Self::MutationError>,
        Infallible: Into<Self::MutationError>,
    {
        let to_remove = self
            .quads()
            .filter_ok(|q| {
                !(ms.matches(q.s()) && mp.matches(q.p()) && mo.matches(q.o()) && mg.matches(q.g()))
            })
            .map_ok(|q| {
                (
                    [
                        q.s().clone_into::<Box<str>>(),
                        q.p().clone_into::<Box<str>>(),
                        q.o().clone_into::<Box<str>>(),
                    ],
                    q.g().map(|g| g.clone_into::<Box<str>>()),
                )
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)?;
        let mut to_remove = to_remove.into_iter().as_quad_source();
        self.remove_all(&mut to_remove)
            .map_err(|err| err.inner_into())?;
        Ok(())
    }
}

/// Marker trait constraining the semantics of
/// [`Dataset`] and [`MutableDataset`].
///
/// It guarantees that
/// (1) quads will never be returned / stored multiple times.
///
/// If the type also implements [`MutableDataset`],
/// it must also ensure that
/// (2) the `bool` or `usize` values returned by [`MutableDataset`]
/// methods accurately describe how many quads were actually added/removed.
///
/// # Note to implementors
/// A type implementing both [`Dataset`] and [`MutableDataset`],
/// enforcing (1) but failing to enforce (2)
/// *must not* implement this trait.
///
/// [`Dataset`]: trait.Dataset.html
/// [`MutableDataset`]: trait.MutableDataset.html

pub trait SetDataset: Dataset {}

#[cfg(test)]
mod test {
    // The code from this module is tested through its use in other modules
    // (especially the macro test_dataset_impl!).
}
