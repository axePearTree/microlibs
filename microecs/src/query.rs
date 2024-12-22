use crate::components::{ComponentsMut, ComponentsRef};
use crate::entities::{Entities, Entity};

/// A trait useful for querying components from a collection.
pub trait Query<'a> {
    type Item: 'a;

    fn iter(self) -> impl Iterator<Item = Option<Self::Item>>;

    fn query(self) -> impl Iterator<Item = Self::Item>
    where
        Self: Sized,
    {
        self.iter().filter_map(|v| v)
    }
}

impl<'a> Query<'a> for &'a Entities<'_> {
    type Item = Entity;

    fn iter(self) -> impl Iterator<Item = Option<Self::Item>> {
        self.0.iter().map(Some)
    }
}

impl<'a, T> Query<'a> for &'a ComponentsRef<'_, T> {
    type Item = &'a T;

    fn iter(self) -> impl Iterator<Item = Option<Self::Item>> {
        self.values.iter()
    }
}

impl<'a, T> Query<'a> for &'a ComponentsMut<'_, T> {
    type Item = &'a T;

    fn iter(self) -> impl Iterator<Item = Option<Self::Item>> {
        self.values.iter()
    }
}

impl<'a, T> Query<'a> for &'a mut ComponentsMut<'_, T> {
    type Item = &'a mut T;

    fn iter(self) -> impl Iterator<Item = Option<Self::Item>> {
        self.values.iter_mut()
    }
}

impl<'a, A, B> Query<'a> for (A, B)
where
    A: Query<'a>,
    B: Query<'a>,
{
    type Item = (A::Item, B::Item);

    fn iter(self) -> impl Iterator<Item = Option<Self::Item>> {
        let (a, b) = self;
        A::iter(a).zip(B::iter(b)).map(|(a, b)| a.zip(b))
    }
}
