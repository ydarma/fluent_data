//! This module defines the neighborhood of some point in space.
//!
//! To get neighbors of a point, use [GetNeighborhood::get_neighborhood] method.

use std::{mem::swap, ops::Deref};

/// A reference to a neighbor and its distance from some point in space.
#[derive(PartialEq, Debug)]
pub struct NeighborDist<Model, RefModel>(RefModel, f64)
where
    RefModel: Deref<Target = Model>;

impl<Point, RefPoint> NeighborDist<Point, RefPoint>
where
    RefPoint: Deref<Target = Point>,
{
    /// The point refenrence
    pub fn coord(&self) -> &Point {
        &self.0
    }

    /// The distance to some other `Point`. Used for testing.
    #[allow(unused)]
    pub fn dist(&self) -> f64 {
        self.1
    }
}

/// The two nearest neighbors of some point in space when they exist.
///
/// The neighborhood consists in two, one or none neighbor.
#[derive(PartialEq, Debug)]
pub enum Neighborhood<Model, RefModel>
where
    RefModel: Deref<Target = Model>,
{
    Two(NeighborDist<Model, RefModel>, NeighborDist<Model, RefModel>),
    One(NeighborDist<Model, RefModel>),
    None,
}

/// Defines a two nearest neighbors getter function.
///
/// This trait is implemented by stucts that represents a set of `Model` in a space of `Point`.
/// ```
/// use fluent_data::{space, neighborhood::{GetNeighborhood, Neighborhood}};
/// fn main() {
///   let points = vec![vec![0.], vec![2.], vec![5.]];
///   let neighborhood = points.iter().get_neighborhood(&vec![3.], space::euclid_dist);
///   if let Neighborhood::Two(n1, n2) = neighborhood {
///       assert_eq!(&points[1], n1.coord());
///       assert_eq!(1., n1.dist());
///       assert_eq!(&points[2], n2.coord());
///       assert_eq!(4., n2.dist()); // square dist
///   } else {
///       panic!()
///   }
/// }
pub trait GetNeighborhood<Point, Model, RefModel, Dist>
where
    Dist: Fn(&Point, &Model) -> f64,
    RefModel: Deref<Target = Model>,
{
    /// Get the two nearest neighbors, ordered by their distance from the given point.
    fn get_neighborhood(&mut self, point: &Point, dist: Dist) -> Neighborhood<Model, RefModel>;
}

/// Implementation of two nearest neighbors getter for an iterator over a set of models.
impl<Iter, Point, Model, RefModel, Dist> GetNeighborhood<Point, Model, RefModel, Dist> for Iter
where
    Iter: Iterator<Item = RefModel>,
    RefModel: Deref<Target = Model>,
    Dist: Fn(&Point, &Model) -> f64,
{
    fn get_neighborhood(&mut self, point: &Point, dist: Dist) -> Neighborhood<Model, RefModel> {
        let iter = self.map(|p| {
            let dist = dist(&point, &p);
            NeighborDist(p, dist)
        });
        fold_0(iter)
    }
}

/// find neighbors given a (model, distance) couples iterator
fn fold_0<Model, RefModel>(
    mut iter: impl Iterator<Item = NeighborDist<Model, RefModel>>,
) -> Neighborhood<Model, RefModel>
where
    RefModel: Deref<Target = Model>,
{
    let p1 = iter.next();
    if let Some(d1) = p1 {
        fold_1(d1, iter)
    } else {
        Neighborhood::None
    }
}

/// find the two nearest neighbors when at least one model exist.
fn fold_1<Model, RefModel>(
    first: NeighborDist<Model, RefModel>,
    mut others: impl Iterator<Item = NeighborDist<Model, RefModel>>,
) -> Neighborhood<Model, RefModel>
where
    RefModel: Deref<Target = Model>,
{
    let p2 = others.next();
    if let Some(d2) = p2 {
        fold_others_2(first, d2, others)
    } else {
        Neighborhood::One(first)
    }
}

/// find the two nearest neighbors when at least two models exist.
fn fold_others_2<Model, RefModel>(
    mut first: NeighborDist<Model, RefModel>,
    mut second: NeighborDist<Model, RefModel>,
    others: impl Iterator<Item = NeighborDist<Model, RefModel>>,
) -> Neighborhood<Model, RefModel>
where
    RefModel: Deref<Target = Model>,
{
    if first.1 > second.1 {
        swap(&mut first, &mut second)
    }
    let (d1, d2) = others.fold((first, second), |(d1, d2), d| smallest(d1, d2, d));
    Neighborhood::Two(d1, d2)
}

/// find the two nearest neighbors among three models.
fn smallest<Model, RefModel>(
    mut d1: NeighborDist<Model, RefModel>,
    mut d2: NeighborDist<Model, RefModel>,
    mut d3: NeighborDist<Model, RefModel>,
) -> (NeighborDist<Model, RefModel>, NeighborDist<Model, RefModel>)
where
    RefModel: Deref<Target = Model>,
{
    if d1.1 > d2.1 {
        swap(&mut d1, &mut d2);
    }
    if d2.1 > d3.1 {
        swap(&mut d2, &mut d3);
    }
    if d1.1 > d2.1 {
        swap(&mut d1, &mut d2);
    }
    (d1, d2)
}

#[cfg(test)]
mod tests {
    use crate::neighborhood::*;
    use crate::space;

    #[test]
    fn test_point_dist() {
        let point = vec![0., 0.];
        let p = NeighborDist(&point, 2.4);
        assert_eq!(&point, p.coord());
        assert_eq!(2.4, p.dist());
    }

    #[test]
    fn test_neighbors() {
        let centers = vec![vec![1., 1.], vec![3.5, -1.6], vec![2.4, 4.], vec![-0.5, 1.]];
        let point = &vec![0., 0.];
        let nn = centers.iter().get_neighborhood(point, space::euclid_dist);
        assert_eq!(
            Neighborhood::Two(
                NeighborDist(&centers[3], 1.25),
                NeighborDist(&centers[0], 2.)
            ),
            nn
        );
        let point = &vec![1.2, 5.];
        let nn = centers.iter().get_neighborhood(point, space::euclid_dist);
        assert_eq!(
            Neighborhood::Two(
                NeighborDist(&centers[2], 2.44),
                NeighborDist(&centers[0], 16.04)
            ),
            nn
        );
    }

    #[test]
    fn test_neighbors_0_model() {
        let centers = vec![];
        let point = &vec![0., 0.];
        let nn = centers.iter().get_neighborhood(point, space::euclid_dist);
        assert_eq!(Neighborhood::None, nn);
    }

    #[test]
    fn test_neighbors_1_model() {
        let centers = vec![vec![1., 1.]];
        let point = &vec![0., 0.];
        let nn = centers.iter().get_neighborhood(point, space::euclid_dist);
        assert_eq!(Neighborhood::One(NeighborDist(&centers[0], 2.)), nn);
    }

    #[test]
    fn test_neighbors_2_models() {
        let centers = vec![vec![1., 1.], vec![-0.5, 1.]];
        let point = &vec![0., 0.];
        let nn = centers.iter().get_neighborhood(point, space::euclid_dist);
        assert_eq!(
            Neighborhood::Two(
                NeighborDist(&centers[1], 1.25),
                NeighborDist(&centers[0], 2.)
            ),
            nn
        );
    }

    #[test]
    fn test_smallest() {
        let p: Vec<f64> = vec![];
        let d1 = NeighborDist(&p, 7.);
        let d2 = NeighborDist(&p, 4.);
        let d3 = NeighborDist(&p, 1.);
        let s = smallest(d1, d2, d3);
        assert_eq!((NeighborDist(&p, 1.), NeighborDist(&p, 4.)), s);
        let d1 = NeighborDist(&p, 7.);
        let d2 = NeighborDist(&p, 4.);
        let d3 = NeighborDist(&p, 5.);
        let s = smallest(d1, d2, d3);
        assert_eq!((NeighborDist(&p, 4.), NeighborDist(&p, 5.)), s);
        let d1 = NeighborDist(&p, 7.);
        let d2 = NeighborDist(&p, 4.);
        let d3 = NeighborDist(&p, 8.);
        let s = smallest(d1, d2, d3);
        assert_eq!((NeighborDist(&p, 4.), NeighborDist(&p, 7.)), s);
    }
}
