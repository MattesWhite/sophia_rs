// this module is transparently re-exported by its parent `graph`
use std::borrow::Borrow;
use std::collections::HashMap;

use ::graph::index::TermIndex;
use ::term::*;
use ::term::factory::TermFactory;

/// An in-memory implemention of [`TermIndex`](../index/trait.TermIndex.html)
/// with `u16` or `u32` as indices.
pub struct TermIndexU<I, F> where
    F: TermFactory,
{
    factory: F,
    next_free: I,
    i2t: Vec<Option<Term<F::Holder>>>,
    i2c: Vec<I>,
    t2i: HashMap<StaticTerm, I>,
}

// Implementation note:
//
// We are fooling the borrow checker by pretending that
// the keys in t2i are StaticTerm,
// while in fact they have a shorter lifetime.
// However, we ensure that keys do not exist longer than the data they borrow
// (inside i2t)...


impl<I, F> TermIndexU<I, F> where
    I: Default,
    F: TermFactory+Default,
{
    pub fn new() -> TermIndexU<I, F> {
        Self::default()
    }
}

impl<I, F> Default for TermIndexU<I, F> where
    I: Default,
    F: TermFactory+Default,
{
    fn default() -> TermIndexU<I, F> {
        TermIndexU {
            factory: F::default(),
            next_free: I::default(),
            i2c: Vec::default(),
            i2t: Vec::default(),
            t2i: HashMap::default(),
        }
    }
}

/// This macro implements TermIndex for TermIndexU<uXX>,
/// where uXX is one of u16, u32...
/// I would prefer to define a generic implementation using traits,
/// but I found this to be non trivial.
macro_rules! impl_term_index {
    ($uXX:ty) => {
        impl<F> TermIndex for TermIndexU<$uXX, F> where
            F: TermFactory+Default,
        {
            type Index = $uXX;
            type Factory = F;

            fn get_index(&self, t: &RefTerm) -> Option<$uXX> {
                self.t2i.get(t).map(|iref| *iref)
            }

            fn make_index(&mut self, t: &RefTerm) -> $uXX {
                let t = self.factory.copy(&t);
                let rt = unsafe { fake_static(&t) };
                if let Some(i) = self.get_index(&rt) {
                    self.i2c[i as usize] += 1;
                    return i;
                }
                self.t2i.insert(rt, self.next_free);
                let i = self.next_free as usize;
                if i == self.i2t.len() {
                    self.next_free += 1;
                    self.i2t.push(Some(t));
                    self.i2c.push(1);
                } else {
                    self.next_free = self.i2c[i];
                    self.i2t[i] = Some(t);
                    self.i2c[i] = 1;
                }
                i as $uXX
            }

            fn get_term(&self, i: $uXX) -> Option<&Term<F::Holder>> {
                let i = i as usize;
                if i < self.i2t.len() {
                    self.i2t[i].as_ref()
                } else {
                    None
                }
            }

            fn inc_ref(&mut self, i: $uXX) {
                let i = i as usize;
                self.i2c[i] += 1;
            }

            fn dec_ref(&mut self, i: $uXX) {
                let i = i as usize;
                self.i2c[i] -= 1;
                if self.i2c[i] == 0 {
                    let t: Term<F::Holder> = self.i2t[i].take().unwrap();
                    self.t2i.remove(unsafe { &fake_static(&t) });
                    self.i2c[i] = self.next_free;
                    self.next_free = i as $uXX;
                }
            }

            fn shrink_to_fit(&mut self) {
                self.factory.shrink_to_fit();
                self.i2c.shrink_to_fit();
                self.i2t.shrink_to_fit();
                self.t2i.shrink_to_fit();
                debug_assert_eq!(self.i2c.len(), self.i2t.len());
            }
        }
    }
}

impl_term_index!(u16);
impl_term_index!(u32);



/// Unsafely converts a term into a StaticTerm.
/// This is to be used *only* when we can guarantee that the produced StaticTerm
/// will not outlive the source term.
/// We use this for keys in TermIndexU::t2i, when the owning term is in TermIndexU::i2t.
#[inline]
unsafe fn fake_static<S, T> (t: &T) -> StaticTerm where
    S: Borrow<str>,
    T: Borrow<Term<S>>,
{
    StaticTerm::from_with(t.borrow(), |txt| &*(txt as *const str))
}



#[cfg(test)]
mod test {
    use super::*;
    use ::term::factory::RcTermFactory;
    use ::graph::index::assert_term_index_works;

    #[test]
    fn test_term_index() {
        let mut ti = TermIndexU::<u16, RcTermFactory>::default();
        assert_term_index_works(&mut ti);
    }

    #[test]
    fn test_term_index_inner() {
        let mut ti = TermIndexU::<u16, RcTermFactory>::default();
        assert_eq!(ti.next_free, 0);
        assert_eq!(ti.i2t.len(), 0);

        use ::ns::rdf;

        assert_eq!(ti.get_index(&rdf::subject), None);
        assert_eq!(ti.make_index(&rdf::subject), 0);
        assert_eq!(ti.get_index(&rdf::subject), Some(0));
        assert_eq!(ti.i2c[0], 1);
        assert_eq!(ti.next_free, 1);
        assert_eq!(ti.i2t.len(), 1);

        assert_eq!(ti.get_index(&rdf::predicate), None);
        assert_eq!(ti.make_index(&rdf::predicate), 1);
        assert_eq!(ti.get_index(&rdf::predicate), Some(1));
        assert_eq!(ti.i2c[1], 1);
        assert_eq!(ti.next_free, 2);
        assert_eq!(ti.i2t.len(), 2);

        assert_eq!(ti.get_index(&rdf::object), None);
        assert_eq!(ti.make_index(&rdf::object), 2);
        assert_eq!(ti.get_index(&rdf::object), Some(2));
        assert_eq!(ti.i2c[2], 1);
        assert_eq!(ti.next_free, 3);
        assert_eq!(ti.i2t.len(), 3);

        assert_eq!(ti.make_index(&rdf::predicate), 1);
        assert_eq!(ti.i2c[1], 2);

        ti.inc_ref(1);
        assert_eq!(ti.i2c[1], 3);

        ti.dec_ref(1);
        assert_eq!(ti.i2c[1], 2);
        assert_eq!(ti.next_free, 3);

        ti.dec_ref(1);
        assert_eq!(ti.i2c[1], 1);
        assert_eq!(ti.next_free, 3);

        ti.dec_ref(1);
        assert_eq!(ti.get_index(&rdf::predicate), None);
        assert_eq!(ti.next_free, 1);
        assert_eq!(ti.i2c[1], 3); // now the previous version of next_free

        ti.dec_ref(0);
        assert_eq!(ti.get_index(&rdf::subject), None);
        assert_eq!(ti.next_free, 0);
        assert_eq!(ti.i2c[0], 1); // now the previous version of next_free

        assert_eq!(ti.make_index(&rdf::type_), 0);
        assert_eq!(ti.i2c[0], 1);
        assert_eq!(ti.i2t.len(), 3);
        assert_eq!(ti.next_free, 1);

        // re-inserting rdf::subject, now ends up in a different place
        assert_eq!(ti.make_index(&rdf::subject), 1);
        assert_eq!(ti.i2c[1], 1);
        assert_eq!(ti.i2t.len(), 3);
        assert_eq!(ti.next_free, 3);

        assert_eq!(ti.make_index(&rdf::Property), 3);
        assert_eq!(ti.i2c[3], 1);
        assert_eq!(ti.i2t.len(), 4);
        assert_eq!(ti.next_free, 4);
    }
}
