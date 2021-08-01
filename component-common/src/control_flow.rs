use chargrid_component::{Component, Ctx, Event, FrameBuffer, Size};
pub struct CF<C: Component>(C);
pub fn cf<C: Component>(component: C) -> CF<C> {
    CF(component)
}

impl<T, C: Component<Output = Option<T>>> Component for CF<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.0.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.0.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.0.size(state, ctx)
    }
}

impl<T, C: Component<Output = Option<T>>> CF<C> {
    pub fn and_then<U, D, F>(self, f: F) -> CF<AndThen<Self, D, F>>
    where
        D: Component<Output = Option<U>, State = C::State>,
        F: FnOnce(T) -> D,
    {
        CF(AndThen::First {
            component: self,
            f: Some(f),
        })
    }
}

pub struct Val<T: Clone>(pub T);
impl<T: Clone> Component for Val<T> {
    type Output = Option<T>;
    type State = ();
    fn render(&self, _state: &Self::State, _ctx: Ctx, _fb: &mut FrameBuffer) {}
    fn update(&mut self, _state: &mut Self::State, _ctx: Ctx, _event: Event) -> Self::Output {
        Some(self.0.clone())
    }
    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        Size::new_u16(0, 0)
    }
}

pub enum AndThen<C, D, F> {
    // f is an option because when it is called, the compiler doesn't know that we're about to
    // destroy it
    First { component: C, f: Option<F> },
    Second(D),
}

impl<T, U, C, D, F> Component for AndThen<C, D, F>
where
    C: Component<Output = Option<T>>,
    D: Component<Output = Option<U>, State = C::State>,
    F: FnOnce(T) -> D,
{
    type Output = Option<U>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        match self {
            Self::First { component, .. } => component.render(state, ctx, fb),
            Self::Second(component) => component.render(state, ctx, fb),
        }
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match self {
            Self::First {
                component,
                ref mut f,
            } => match component.update(state, ctx, event) {
                None => None,
                Some(t) => {
                    let mut d = (f.take().unwrap())(t);
                    if let Some(u) = d.update(state, ctx, Event::Peek) {
                        Some(u)
                    } else {
                        *self = Self::Second(d);
                        None
                    }
                }
            },
            Self::Second(component) => component.update(state, ctx, event),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        match self {
            Self::First { component, .. } => component.size(state, ctx),
            Self::Second(component) => component.size(state, ctx),
        }
    }
}

pub struct Map<C, F> {
    component: C,
    f: F,
}

impl<T, U, C, F> Component for Map<C, F>
where
    C: Component<Output = Option<T>>,
    F: FnMut(T) -> U,
{
    type Output = Option<U>;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        match self.component.update(state, ctx, event) {
            None => None,
            Some(t) => Some((self.f)(t)),
        }
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}

#[macro_export]
macro_rules! mkeither {
    ($type:ident = $first:ident | $($rest:ident)|*) => {
        pub enum $type<$first, $($rest),*> {
            $first($first),
            $($rest($rest)),*
        }
        impl<$first, $($rest),*> Component for $type<$first, $($rest),*>
            where
                $first: Component,
                $($rest: Component<Output = $first::Output, State = $first::State>),*
        {
            type Output = $first::Output;
            type State = $first::State;

            fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
                match self {
                    $type::$first(x) => x.render(state, ctx, fb),
                    $($type::$rest(x) => x.render(state, ctx, fb),),*
                }
            }
            fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
                match self {
                    $type::$first(x) => x.update(state, ctx, event),
                    $($type::$rest(x) => x.update(state, ctx, event),),*
                }
            }
            fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
                match self {
                    $type::$first(x) => x.size(state, ctx),
                    $($type::$rest(x) => x.size(state, ctx),),*
                }
            }
        }
    };
}

pub use crate::mkeither;
