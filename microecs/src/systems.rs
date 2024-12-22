use crate::components::{ComponentsMut, ComponentsRef};
use crate::entities::Entities;
use crate::prelude::Resources;
use crate::resources::{ItemMut, ItemRef, ResourceMut, ResourceRef};
use crate::{Chunk, CommandQueue, Commands, Error};

pub struct SystemsContext<'a> {
    chunk: &'a mut Chunk,
    resources: &'a mut Resources,
    command_queue: &'a mut CommandQueue,
}

impl<'a> SystemsContext<'a> {
    pub fn new(
        chunk: &'a mut Chunk,
        resources: &'a mut Resources,
        command_queue: &'a mut CommandQueue,
    ) -> Self {
        Self {
            chunk,
            command_queue,
            resources,
        }
    }

    pub fn run<F, P>(&mut self, mut system_function: F) -> Result<&mut Self, Error>
    where
        F: System<P>,
    {
        let params = F::get_params(&self.chunk, &self.resources, &self.command_queue)?;
        system_function.run(params);
        self.command_queue.flush(self.chunk, self.resources)?;
        Ok(self)
    }
}

pub trait System<Params> {
    type Params<'a>;

    fn get_params<'a>(
        chunk: &'a Chunk,
        resources: &'a Resources,
        command_queue: &'a CommandQueue,
    ) -> Result<Self::Params<'a>, Error>;

    fn run(&mut self, params: Self::Params<'_>);
}

pub trait SystemParam {
    type Param<'a>;

    fn get_param<'a>(
        chunk: &'a Chunk,
        resources: &'a Resources,
        command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error>;
}

impl SystemParam for Entities<'_> {
    type Param<'a> = Entities<'a>;

    fn get_param<'a>(
        chunk: &'a Chunk,
        _resources: &'a Resources,
        _command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        Ok(Entities(&chunk.entities))
    }
}

impl<T> SystemParam for ComponentsRef<'_, T>
where
    T: 'static,
{
    type Param<'a> = ComponentsRef<'a, T>;

    fn get_param<'a>(
        chunk: &'a Chunk,
        _resources: &'a Resources,
        _command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        chunk.components_ref()
    }
}

impl<T> SystemParam for ComponentsMut<'_, T>
where
    T: 'static,
{
    type Param<'a> = ComponentsMut<'a, T>;

    fn get_param<'a>(
        chunk: &'a Chunk,
        _resources: &'a Resources,
        _command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        chunk.components_mut()
    }
}

impl SystemParam for Commands<'_> {
    type Param<'a> = Commands<'a>;

    fn get_param<'a>(
        _chunk: &'a Chunk,
        _resources: &'a Resources,
        command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        command_queue.deferred_commands()
    }
}

impl<T> SystemParam for ResourceRef<'_, T>
where
    T: 'static,
{
    type Param<'a> = ResourceRef<'a, T>;

    fn get_param<'a>(
        _chunk: &'a Chunk,
        resources: &'a Resources,
        _command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        resources.resource_ref::<T>()
    }
}

impl<T> SystemParam for ResourceMut<'_, T>
where
    T: 'static,
{
    type Param<'a> = ResourceMut<'a, T>;

    fn get_param<'a>(
        _chunk: &'a Chunk,
        resources: &'a Resources,
        _command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        resources.resource_mut::<T>()
    }
}


impl<T> SystemParam for ItemRef<'_, T>
where
    T: 'static,
{
    type Param<'a> = ItemRef<'a, T>;

    fn get_param<'a>(
        chunk: &'a Chunk,
        _resources: &'a Resources,
        _command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        Ok(chunk.items.resource_ref::<T>()?.into_item())
    }
}

impl<T> SystemParam for ItemMut<'_, T>
where
    T: 'static,
{
    type Param<'a> = ItemMut<'a, T>;

    fn get_param<'a>(
        chunk: &'a Chunk,
        _resources: &'a Resources,
        _command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        Ok(chunk.items.resource_mut::<T>()?.into_item_mut())
    }
}

impl<A, B> SystemParam for (A, B)
where
    A: SystemParam,
    B: SystemParam,
{
    type Param<'a> = (A::Param<'a>, B::Param<'a>);

    fn get_param<'a>(
        chunk: &'a Chunk,
        resources: &'a Resources,
        command_queue: &'a CommandQueue,
    ) -> Result<Self::Param<'a>, Error> {
        Ok((
            A::get_param(chunk, resources, command_queue)?,
            B::get_param(chunk, resources, command_queue)?,
        ))
    }
}

// rustc: we have variadics at home
// variadics at home:
macro_rules! impl_traits_for_tuple {
    ( $($T:ident),+ ) => {
        impl<Func, $($T),+> System<($($T,)+)> for Func
        where
            Func: FnMut($($T,)+),
            Func: for<'a> FnMut($($T::Param<'a>,)+),
            $($T: SystemParam,)+
        {
            type Params<'a> = ($($T::Param<'a>,)+);

            fn get_params<'a>(
                chunk: &'a Chunk,
                resources: &'a Resources,
                command_queue: &'a CommandQueue,
            ) -> Result<Self::Params<'a>, Error> {
                Ok(($($T::get_param(chunk, resources, command_queue)?,)+))
            }

            fn run(&mut self, params: Self::Params<'_>) {
                #[allow(non_snake_case)]
                let ($($T,)+) = params;
                self($($T,)+)
            }
        }
    };
}

impl_traits_for_tuple!(Param1);
impl_traits_for_tuple!(Param1, Param2);
impl_traits_for_tuple!(Param1, Param2, Param3);
impl_traits_for_tuple!(Param1, Param2, Param3, Param4);
impl_traits_for_tuple!(Param1, Param2, Param3, Param4, Param5);
impl_traits_for_tuple!(Param1, Param2, Param3, Param4, Param5, Param6);
impl_traits_for_tuple!(Param1, Param2, Param3, Param4, Param5, Param6, Param7);
impl_traits_for_tuple!(Param1, Param2, Param3, Param4, Param5, Param6, Param7, Param8);
impl_traits_for_tuple!(Param1, Param2, Param3, Param4, Param5, Param6, Param7, Param8, Param9);
#[rustfmt::skip]
impl_traits_for_tuple!(Param1, Param2, Param3, Param4, Param5, Param6, Param7, Param8, Param9, Param10);
#[rustfmt::skip]
impl_traits_for_tuple!(Param1, Param2, Param3, Param4, Param5, Param6, Param7, Param8, Param9, Param10, Param11);
#[rustfmt::skip]
impl_traits_for_tuple!(Param1, Param2, Param3, Param4, Param5, Param6, Param7, Param8, Param9, Param10, Param11, Param12);
