//! Contains helper functions and macros for testing Dataset implementations

use std::fmt::Debug;

use crate::dataset::*;
use crate::graph::test::*;
use crate::ns::*;
use crate::quad::stream::*;
use crate::quad::streaming_mode::{QuadStreamingMode, UnsafeQuad};
use crate::quad::*;
use lazy_static::lazy_static;
use sophia_term::*;

lazy_static! {
    pub static ref G1: StaticTerm = StaticTerm::new_iri_suffixed(NS, "G1").unwrap();
    pub static ref G2: StaticTerm = StaticTerm::new_iri_suffixed(NS, "G2").unwrap();
    //
    pub static ref DG: Option<&'static StaticTerm> = None;
    pub static ref GN1: Option<&'static StaticTerm> = Some(&G1);
    pub static ref GN2: Option<&'static StaticTerm> = Some(&G2);
}

pub fn no_quad() -> impl QuadSource {
    let v = Vec::<([StaticTerm; 3], Option<StaticTerm>)>::new();
    v.into_iter().as_quad_source()
}

pub fn some_quads() -> impl QuadSource {
    let v = vec![
        ([*C1, rdf::type_, rdfs::Class], *DG),
        ([*C1, rdf::type_, rdfs::Class], *GN1),
        ([*C2, rdf::type_, rdfs::Class], *DG),
        ([*C2, rdfs::subClassOf, *C1], *GN1),
        ([*P1, rdf::type_, rdf::Property], *DG),
        ([*P1, rdfs::domain, *C1], *GN1),
        ([*P1, rdfs::range, *C2], *GN1),
        ([*P2, rdf::type_, rdf::Property], *DG),
        ([*P2, rdfs::domain, *C2], *GN1),
        ([*P2, rdfs::range, *C2], *GN1),
        ([*I1A, rdf::type_, *C1], *GN2),
        ([*I1B, rdf::type_, *C1], *GN2),
        ([*I2A, rdf::type_, *C2], *GN2),
        ([*I2B, rdf::type_, *C2], *GN2),
        ([*I1A, *P1, *I2A], *GN2),
        ([*I1B, *P1, *I2B], *GN2),
        ([*I2A, *P2, *I2B], *GN2),
    ];

    v.into_iter().as_quad_source()
}

pub fn strict_node_types_quads() -> impl QuadSource {
    vec![
        ([rdf::type_, rdf::type_, rdf::Property], Some(&rdf::type_)),
        ([*B1, rdf::type_, *L1], Some(&B2)),
        ([*B2, rdf::type_, *B1], None),
        ([*B2, rdf::type_, *L2], None),
        ([*B2, rdf::type_, *L2E], None),
    ]
    .into_iter()
    .as_quad_source()
}

pub fn generalized_node_types_quads() -> impl QuadSource {
    vec![
        ([rdf::type_, rdf::type_, rdf::Property], Some(&rdf::type_)),
        ([*B1, *B2, *B1], Some(&B2)),
        ([*L2, *L1, *L1], Some(&L2)),
        ([*V1, *V2, *V3], Some(&V3)),
        ([*B2, *V1, *L2E], None),
    ]
    .into_iter()
    .as_quad_source()
}

pub fn as_box_q<Q: Quad, E>(quad: Result<Q, E>) -> ([BoxTerm; 3], Option<BoxTerm>)
where
    E: Debug,
{
    let quad = quad.unwrap();
    (
        [
            quad.s().clone_into(),
            quad.p().clone_into(),
            quad.o().clone_into(),
        ],
        quad.g().map(|n| n.clone_into()),
    )
}

#[allow(dead_code)]
pub fn dump_dataset<D: Dataset>(d: &D)
where
    <<D::Quad as QuadStreamingMode>::UnsafeQuad as UnsafeQuad>::TermData: Debug,
{
    println!("<<<<");
    for q in d.quads() {
        let q = q.unwrap();
        println!("{:?}\n{:?}\n{:?}\n{:?}\n\n", q.s(), q.p(), q.o(), q.g());
    }
    println!(">>>>");
}

pub fn make_quad_source() -> impl QuadSource {
    vec![
        [&*C1, &rdf::type_, &rdfs::Class, &G1],
        [&*C1, &rdfs::subClassOf, &*C2, &G1],
    ]
    .into_iter()
    .as_quad_source()
}

/// Generates a test suite for implementations of
/// [`Dataset`], [`CollectibleDataset`] and [`MutableDataset`].
///
/// If your type only implements [`Dataset`] and [`CollectibleDataset`],
/// you should use [`test_immutable_dataset_impl`] instead.
///
/// This macro is only available when the feature `test_macros` is enabled.
///
/// It accepts the following parameters:
/// * `module_name`: the name of the module to generate (defaults to `test`);
/// * `dataset_impl`: the type to test, implementing [`Dataset`], [`CollectibleDataset`] and [`MutableDataset`];
/// * `is_set`: a boolean, indicating if `dataset_impl` implements [`SetDataset`]
///   (defaults to `true`);
/// * `is_gen`: a boolean, indicating if `dataset_impl` supports the [generalized model]
///   (defaults to `true`).
/// * `dataset_collector`: a function used to create an empy instance of `dataset_impl`
///   (defaults to `dataset_impl::from_quad_source`);
/// * `mt` is used internally, do not touch it...
///
/// [`Dataset`]: dataset/trait.Dataset.html
/// [`CollectibleDataset`]: dataset/trait.CollectibleDataset.html
/// [`MutableDataset`]: dataset/trait.MutableDataset.html
/// [`test_immutable_dataset_impl`]: ./macro.test_immutable_dataset_impl
/// [`SetDataset`]: dataset/trait.SetDataset.html
/// [generalized model]: ./index.html
#[macro_export]
macro_rules! test_dataset_impl {
    ($dataset_impl: ident) => {
        test_dataset_impl!(test, $dataset_impl);
    };
    ($module_name: ident, $dataset_impl: ident) => {
        test_dataset_impl!($module_name, $dataset_impl, true);
    };
    ($module_name: ident, $dataset_impl: ident, $is_set: expr) => {
        test_dataset_impl!($module_name, $dataset_impl, $is_set, true);
    };
    ($module_name: ident, $dataset_impl: ident, $is_set: expr, $is_gen: expr) => {
        test_dataset_impl!($module_name, $dataset_impl, $is_set, $is_gen, $dataset_impl::from_quad_source);
    };
    ($module_name: ident, $dataset_impl: ident, $is_set: expr, $is_gen: expr, $dataset_collector: path) => {
        test_dataset_impl!($module_name, $dataset_impl, $is_set, $is_gen, $dataset_collector, {
            // these tests will only be performed for implementations of `MutableDataset`
            #[test]
            fn test_simple_mutations() -> MDResult<$dataset_impl, ()> {
                let mut d = $dataset_collector(no_quad()).unwrap();
                assert_eq!(d.quads().count(), 0);
                assert!(MutableDataset::insert(
                    &mut d,
                    &C1,
                    &rdf::type_,
                    &rdfs::Class,
                    *DG
                )?);
                assert_eq!(d.quads().count(), 1);
                assert!(MutableDataset::insert(
                    &mut d,
                    &C1,
                    &rdfs::subClassOf,
                    &C2,
                    *GN1
                )?);
                assert_eq!(d.quads().count(), 2);
                assert!(MutableDataset::remove(
                    &mut d,
                    &C1,
                    &rdf::type_,
                    &rdfs::Class,
                    *DG
                )?);
                assert_eq!(d.quads().count(), 1);
                assert!(MutableDataset::remove(
                    &mut d,
                    &C1,
                    &rdfs::subClassOf,
                    &C2,
                    *GN1
                )?);
                assert_eq!(d.quads().count(), 0);
                Ok(())
            }

            #[test]
            fn test_no_duplicate() -> MDResult<$dataset_impl, ()> {
                if $is_set {
                    let mut d = $dataset_collector(no_quad()).unwrap();
                    assert_eq!(d.quads().count(), 0);
                    assert!(MutableDataset::insert(
                        &mut d,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *DG
                    )?);
                    assert_eq!(d.quads().count(), 1);
                    assert!(!MutableDataset::insert(
                        &mut d,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *DG
                    )?);
                    assert_eq!(d.quads().count(), 1);
                    assert!(MutableDataset::remove(
                        &mut d,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *DG
                    )?);
                    assert_eq!(d.quads().count(), 0);
                    assert!(!MutableDataset::remove(
                        &mut d,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *DG
                    )?);
                    assert_eq!(d.quads().count(), 0);
                } else {
                    println!("effectively skipped, since is_set is false");
                }
                Ok(())
            }

            #[test]
            fn test_different_graphs_do_not_count_as_duplicate() -> MDResult<$dataset_impl, ()> {
                let mut d = $dataset_collector(no_quad()).unwrap();
                assert_eq!(d.quads().count(), 0);
                assert!(MutableDataset::insert(
                    &mut d,
                    &C1,
                    &rdf::type_,
                    &rdfs::Class,
                    *DG
                )?);
                assert_eq!(d.quads().count(), 1);
                assert!(MutableDataset::insert(
                    &mut d,
                    &C1,
                    &rdf::type_,
                    &rdfs::Class,
                    *GN1
                )?);
                assert_eq!(d.quads().count(), 2);
                assert!(MutableDataset::remove(
                    &mut d,
                    &C1,
                    &rdf::type_,
                    &rdfs::Class,
                    *DG
                )?);
                assert_eq!(d.quads().count(), 1);
                assert!(MutableDataset::remove(
                    &mut d,
                    &C1,
                    &rdf::type_,
                    &rdfs::Class,
                    *GN1
                )?);
                assert_eq!(d.quads().count(), 0);
                Ok(())
            }

            #[test]
            fn test_x_all_mutations() {
                let mut d = $dataset_collector(no_quad()).unwrap();
                assert_eq!(d.quads().count(), 0);
                assert_eq!(d.insert_all(make_quad_source()).unwrap(), 2);
                assert_eq!(d.quads().count(), 2, "after insert_all");
                if $is_set {
                    assert_eq!(d.insert_all(make_quad_source()).unwrap(), 0);
                    assert_eq!(d.quads().count(), 2, "after insert_all again");
                }
                assert_eq!(d.remove_all(make_quad_source()).unwrap(), 2);
                assert_eq!(d.quads().count(), 0, "after remove_all");
                assert_eq!(d.remove_all(make_quad_source()).unwrap(), 0);
                assert_eq!(d.quads().count(), 0, "after remove_all again");
            }

            #[test]
            fn test_remove_matching() -> MDResult<$dataset_impl, ()> {
                let mut d = $dataset_collector(some_quads()).unwrap();

                let o_matcher = [C1.clone(), C2.clone()];
                d.remove_matching(&ANY, &rdf::type_, &o_matcher[..], &ANY)?;
                assert_consistent_hint(13, d.quads().size_hint());
                Ok(())
            }

            #[test]
            fn test_retain_matching() -> MDResult<$dataset_impl, ()> {
                let mut d = $dataset_collector(some_quads()).unwrap();

                let o_matcher = [C1.clone(), C2.clone()];
                d.retain_matching(&ANY, &rdf::type_, &o_matcher[..], &ANY)?;
                print!("{:?}", d.quads().size_hint());
                assert_consistent_hint(4, d.quads().size_hint());
                Ok(())
            }
        });
    };
    ($module_name: ident, $dataset_impl: ident, $is_set: expr, $is_gen: expr, $dataset_collector: path, { $($mt:tt)* }) => {
        #[cfg(test)]
        mod $module_name {
            use sophia_term::matcher::ANY;
            use $crate::dataset::test::*;
            use $crate::dataset::*;
            use $crate::graph::test::*;
            use $crate::ns::*;

            #[allow(unused_imports)]
            use super::*;

            #[test]
            fn test_quads() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads();
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&ANY, &ANY, &ANY, &ANY)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), d.quads().count());
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *GN1)?);
                    assert!(!Dataset::contains(&v, &P1, &rdf::type_, &rdfs::Class, *DG)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_s() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_s(&C2);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&*C2, &ANY, &ANY, &ANY)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 2);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(
                        &v,
                        &C2,
                        &rdf::type_,
                        &rdfs::Class,
                        *GN1
                    )?);
                    assert!(!Dataset::contains(
                        &v,
                        &C2,
                        &rdf::type_,
                        &rdf::Property,
                        *DG
                    )?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_p() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_p(&rdfs::subClassOf);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&ANY, &rdfs::subClassOf, &ANY, &ANY)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 1);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *GN1)?);
                    assert!(!Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *DG)?);
                    assert!(!Dataset::contains(
                        &v,
                        &C2,
                        &rdfs::subClassOf,
                        &rdfs::Class,
                        *DG
                    )?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_o() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_o(&I2B);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&ANY, &ANY, &*I2B, &ANY)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 2);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &I1B, &P1, &I2B, *GN2)?);
                    assert!(!Dataset::contains(&v, &I1B, &P1, &I2B, *GN1)?);
                    assert!(!Dataset::contains(&v, &I2A, &P1, &I2B, *GN2)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_g() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_g(*GN1);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&ANY, &ANY, &ANY, &*GN1)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 6);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *GN1)?);
                    assert!(!Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_sp() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_sp(&C2, &rdf::type_);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&*C2, &rdf::type_, &ANY, &ANY)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 1);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(
                        &v,
                        &C2,
                        &rdf::type_,
                        &rdfs::Class,
                        *GN1
                    )?);
                    assert!(!Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_so() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_so(&C2, &C1);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&*C2, &ANY, &*C1, &ANY)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 1);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *GN1)?);
                    assert!(!Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *DG)?);
                    assert!(!Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_po() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_po(&rdf::type_, &rdfs::Class);
                let hint = quads.size_hint();
                for iter in vec![
                    quads,
                    d.quads_matching(&ANY, &rdf::type_, &rdfs::Class, &ANY),
                ] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 3);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(
                        &v,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *GN2
                    )?);
                    assert!(!Dataset::contains(
                        &v,
                        &P1,
                        &rdf::type_,
                        &rdf::Property,
                        *DG
                    )?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_sg() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_sg(&C2, *GN1);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&*C2, &ANY, &ANY, &*GN1)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 1);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *GN1)?);
                    assert!(!Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *DG)?);
                    assert!(!Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_pg() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_pg(&rdf::type_, *GN1);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&ANY, &rdf::type_, &ANY, &*GN1)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 1);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *GN1)?);
                    assert!(!Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *GN1)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_og() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_og(&C1, *GN1);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&ANY, &ANY, &*C1, &*GN1)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 2);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *GN1)?);
                    assert!(!Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *DG)?);
                    assert!(!Dataset::contains(&v, &I1A, &rdf::type_, &C1, *GN2)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_spo() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_spo(&C1, &rdf::type_, &rdfs::Class);
                let hint = quads.size_hint();
                for iter in vec![
                    quads,
                    d.quads_matching(&*C1, &rdf::type_, &rdfs::Class, &ANY),
                ] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 2);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *GN1)?);
                    assert!(!Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_spg() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_spg(&C1, &rdf::type_, *DG);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&*C1, &rdf::type_, &ANY, &*DG)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 1);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(
                        &v,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *GN1
                    )?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_sog() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_sog(&C1, &rdfs::Class, *DG);
                let hint = quads.size_hint();
                for iter in vec![quads, d.quads_matching(&*C1, &ANY, &rdfs::Class, &*DG)] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 1);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(
                        &v,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *GN1
                    )?);
                    assert!(!Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_pog() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_pog(&rdf::type_, &rdfs::Class, *DG);
                let hint = quads.size_hint();
                for iter in vec![
                    quads,
                    d.quads_matching(&ANY, &rdf::type_, &rdfs::Class, &*DG),
                ] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 2);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(
                        &v,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *GN1
                    )?);
                }
                Ok(())
            }

            #[test]
            fn test_quads_with_spog() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let quads = d.quads_with_spog(&C1, &rdf::type_, &rdfs::Class, *DG);
                let hint = quads.size_hint();
                for iter in vec![
                    quads,
                    d.quads_matching(&*C1, &rdf::type_, &rdfs::Class, &*DG),
                ] {
                    let v: Vec<_> = iter.map(as_box_q).collect();
                    assert_eq!(v.len(), 1);
                    assert_consistent_hint(v.len(), hint);
                    assert!(Dataset::contains(&v, &C1, &rdf::type_, &rdfs::Class, *DG)?);
                    assert!(!Dataset::contains(
                        &v,
                        &C1,
                        &rdf::type_,
                        &rdfs::Class,
                        *GN1
                    )?);
                    assert!(!Dataset::contains(&v, &C2, &rdf::type_, &rdfs::Class, *DG)?);
                }
                Ok(())
            }

            #[test]
            fn test_contains() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();
                assert!(Dataset::contains(&d, &C2, &rdfs::subClassOf, &C1, *GN1)?);
                assert!(!Dataset::contains(&d, &C1, &rdfs::subClassOf, &C2, *GN1)?);
                Ok(())
            }

            #[test]
            fn test_quads_matching() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let p_matcher: [StaticTerm; 2] = [rdf::type_.clone(), rdfs::domain.clone()];
                let o_matcher: [StaticTerm; 2] = [C1.clone(), C2.clone()];
                let g_matcher = |g: Option<&Term<&str>>| g.is_some();
                let v: Vec<_> = d
                    .quads_matching(&ANY, &p_matcher[..], &o_matcher[..], &g_matcher)
                    .map(as_box_q)
                    .collect();
                assert_eq!(v.len(), 6);
                assert!(Dataset::contains(&v, &P1, &rdfs::domain, &C1, *GN1)?);
                assert!(Dataset::contains(&v, &P2, &rdfs::domain, &C2, *GN1)?);
                assert!(Dataset::contains(&v, &I1A, &rdf::type_, &C1, *GN2)?);
                assert!(Dataset::contains(&v, &I2A, &rdf::type_, &C2, *GN2)?);
                assert!(!Dataset::contains(&v, &C2, &rdfs::subClassOf, &C1, *GN1)?);
                assert!(!Dataset::contains(
                    &v,
                    &C1,
                    &rdf::type_,
                    &rdfs::Class,
                    *GN1
                )?);
                Ok(())
            }

            #[test]
            fn test_subjects() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let subjects = d.subjects().unwrap();
                assert_eq!(subjects.len(), 8);

                let rsubjects: std::collections::HashSet<_> =
                    subjects.iter().map(|t| t.as_ref_str()).collect();
                assert!(rsubjects.contains(&C1));
                assert!(rsubjects.contains(&C2));
                assert!(rsubjects.contains(&P1));
                assert!(rsubjects.contains(&P2));
                assert!(rsubjects.contains(&I1A));
                assert!(rsubjects.contains(&I1B));
                assert!(rsubjects.contains(&I2A));
                assert!(rsubjects.contains(&I2B));
                Ok(())
            }

            #[test]
            fn test_predicates() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let predicates = d.predicates().unwrap();
                assert_eq!(predicates.len(), 6);

                let rpredicates: std::collections::HashSet<_> =
                    predicates.iter().map(|t| t.as_ref_str()).collect();
                assert!(rpredicates.contains(&rdf::type_));
                assert!(rpredicates.contains(&rdfs::subClassOf));
                assert!(rpredicates.contains(&rdfs::domain));
                assert!(rpredicates.contains(&rdfs::range));
                assert!(rpredicates.contains(&P1));
                assert!(rpredicates.contains(&P2));
                Ok(())
            }

            #[test]
            fn test_objects() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let objects = d.objects().unwrap();
                assert_eq!(objects.len(), 6);

                let robjects: std::collections::HashSet<_> =
                    objects.iter().map(|t| t.as_ref_str()).collect();
                assert!(robjects.contains(&rdf::Property));
                assert!(robjects.contains(&rdfs::Class));
                assert!(robjects.contains(&C1));
                assert!(robjects.contains(&C2));
                assert!(robjects.contains(&I2A));
                assert!(robjects.contains(&I2B));
                Ok(())
            }

            #[test]
            fn test_graph_names() -> MDResult<$dataset_impl, ()> {
                let d = $dataset_collector(some_quads()).unwrap();

                let graph_names = d.graph_names().unwrap();
                assert_eq!(graph_names.len(), 2);

                let rgraph_names: std::collections::HashSet<_> =
                    graph_names.iter().map(|t| t.as_ref_str()).collect();
                assert!(rgraph_names.contains(&G1));
                assert!(rgraph_names.contains(&G2));
                Ok(())
            }

            #[test]
            fn test_iris() -> MDResult<$dataset_impl, ()> {
                let d = if $is_gen {
                    $dataset_collector(generalized_node_types_quads()).unwrap()
                } else {
                    $dataset_collector(strict_node_types_quads()).unwrap()
                };

                let iris = d.iris().unwrap();
                assert_eq!(iris.len(), 2);

                let riris: std::collections::HashSet<_> =
                    iris.iter().map(|t| t.as_ref_str()).collect();
                assert!(riris.contains(&rdf::Property));
                assert!(riris.contains(&rdf::type_));
                Ok(())
            }

            #[test]
            fn test_bnodes() -> MDResult<$dataset_impl, ()> {
                let d = if $is_gen {
                    $dataset_collector(generalized_node_types_quads()).unwrap()
                } else {
                    $dataset_collector(strict_node_types_quads()).unwrap()
                };

                let bnodes = d.bnodes().unwrap();
                assert_eq!(bnodes.len(), 2);

                let rbnodes: std::collections::HashSet<_> =
                    bnodes.iter().map(|t| t.value()).collect();
                assert!(rbnodes.contains("1"));
                assert!(rbnodes.contains("2"));
                Ok(())
            }

            #[test]
            fn test_literals() -> MDResult<$dataset_impl, ()> {
                let d = if $is_gen {
                    $dataset_collector(generalized_node_types_quads()).unwrap()
                } else {
                    $dataset_collector(strict_node_types_quads()).unwrap()
                };

                let literals = d.literals().unwrap();
                assert_eq!(literals.len(), 3);

                let rliterals: std::collections::HashSet<_> =
                    literals.iter().map(|t| t.as_ref_str()).collect();
                assert!(rliterals.contains(&StaticTerm::from("lit1")));
                assert!(rliterals.contains(&StaticTerm::from("lit2")));
                assert!(rliterals.contains(&StaticTerm::new_literal_lang("lit2", "en").unwrap()));
                Ok(())
            }

            #[test]
            fn test_variables() -> MDResult<$dataset_impl, ()> {
                if $is_gen {
                    let d = $dataset_collector(generalized_node_types_quads()).unwrap();

                    let variables = d.variables().unwrap();
                    assert_eq!(variables.len(), 3);

                    let rvariables: std::collections::HashSet<_> =
                        variables.iter().map(|t| t.value()).collect();
                    assert!(rvariables.contains("v1"));
                    assert!(rvariables.contains("v2"));
                    assert!(rvariables.contains("v3"));
                } else {
                    let d = $dataset_collector(strict_node_types_quads()).unwrap();

                    let variables = d.variables().unwrap();
                    assert_eq!(variables.len(), 0);
                }
                Ok(())
            }

            // Tests for MutableGraph only, if enabled:
            $($mt)*
        }
    };
}

/// Generates a test suite for implementations of
/// [`Dataset`], [`CollectibleDataset`].
///
/// If your type also implements [`MutableDataset`],
/// you should use [`test_dataset_impl`] instead.
///
/// This macro is only available when the feature `test_macros` is enabled.
///
/// It accepts the following parameters:
/// * `module_name`: the name of the module to generate (defaults to `test`);
/// * `dataset_impl`: the type to test, implementing [`Dataset`] and [`CollectibleDataset`];
/// * `is_set`: a boolean, indicating if `dataset_impl` implements [`SetDataset`]
///   (defaults to `true`);
/// * `is_gen`: a boolean, indicating if `dataset_impl` supports the [generalized model]
///   (defaults to `true`);
/// * `dataset_collector`: a function used to collect quads into an instance of `dataset_impl`
///   (defaults to `dataset_impl::from_quad_source`);
///
/// [`Dataset`]: dataset/trait.Dataset.html
/// [`CollectibleDataset`]: dataset/trait.CollectibleDataset.html
/// [`MutableDataset`]: dataset/trait.MutableDataset.html
/// [`test_dataset_impl`]: ./macro.test_dataset_impl
/// [`SetDataset`]: dataset/trait.SetDataset.html
/// [generalized model]: ./index.html
#[macro_export]
macro_rules! test_immutable_dataset_impl {
    ($dataset_impl: ident) => {
        test_immutable_dataset_impl!(test, $dataset_impl);
    };
    ($module_name: ident, $dataset_impl: ident) => {
        test_immutable_dataset_impl!($module_name, $dataset_impl, true);
    };
    ($module_name: ident, $dataset_impl: ident, $is_set: expr) => {
        test_immutable_dataset_impl!($module_name, $dataset_impl, $is_set, true);
    };
    ($module_name: ident, $dataset_impl: ident, $is_set: expr, $is_gen: expr) => {
        test_immutable_dataset_impl!(
            $module_name,
            $dataset_impl,
            $is_set,
            $is_gen,
            $dataset_impl::from_quad_source
        );
    };
    ($module_name: ident, $dataset_impl: ident, $is_set: expr, $is_gen: expr, $dataset_collector: path) => {
        // calling test_dataset_impl, but passing an empty block as mt (the mutability tests)
        test_dataset_impl!(
            $module_name,
            $dataset_impl,
            $is_set,
            $is_gen,
            $dataset_collector,
            {}
        );
    };
}
