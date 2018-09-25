//! This module defines traits the user needs to implement a Ruukh Component.
//!
//! The main traits of concern for a user are [Lifecycle](trait.Lifecycle.html)
//! and [Render](trait.Render.html).
//!
//! The other trait which is required is the uber important
//! [Component](trait.Component.html) trait, which is automatically implemented
//! by using the `#[component]` attribute on top of the component struct. You
//! are advised against to implement it yourself other than to learn how
//! everything wires up.
//!
//! So onto implementing the `Lifecycle` and `Render` trait. By the way if you
//! desire not to write the implementation of `Lifecycle` yourself, you may
//! write `#[derive(Lifecycle)]` on your component struct.
//!
//! # Example
//! ```ignore,compile_fail
//! #[component]
//! struct Help;
//!
//! impl Lifecycle for Help {}
//!
//! impl Render for Help {
//!     fn render(&self) -> Markup<Self> {
//!         html! {
//!             <div>
//!                 This is help section
//!             </div>
//!             <div>
//!                 There are a lot of divs here. Here comes a <button>Button</button>
//!             </div>
//!         }
//!     }
//! }
//! ```
//!
//! The lifecycle implementation provided here is bare bones. Actually,
//! lifecycle provides a default implementation which does nothing. If you want
//! to provide a much more specific implementation, you can selectively
//! implement its methods.
//!
//! # Example
//! ```ignore,compile_fail
//! impl Lifecycle for Help {
//!     fn created(&self) {
//!         println!("It got created just now!");
//!     }
//! }
//! ```
//!
//! Note: Docs on component macros are located
//! [here](../../ruukh_codegen/index.html).

use crate::{Markup, MessageSender, Shared};

/// Trait to define a component. You do not need to implement this trait. Auto
/// implement this trait by using `#[component]` on a component struct (which
/// also does other magical stuff).
///
/// ## Internals
///
/// A component struct which implements this trait usually has the following
/// fields defined into it.
/// 1. Prop fields - The fields which are used by their parent to pass values
/// into it.
/// 2. State fields - The fields which are used internally by the component
/// itself to manage its state.
/// 3. Event field - The field which stores the `Self::Events` value which are
/// passed by the parent to be invoked when the defined events occur.
/// 4. Status field: The field which stores the state metadata i.e. stores the
/// mutable state itself, whether state/props are dirty as well as state change
/// notifying mechanism.
pub trait Component: 'static {
    /// The prop type of a Component.
    ///
    /// ## Internals
    ///
    /// The props type is generated by copying all the prop fields from a
    /// `#[component]` struct. This type is named by concatenating component
    /// name with `Props`.
    type Props;
    /// The event type of a Component.
    ///
    /// ## Internals
    ///
    /// The events type is generated by parsing all the event signatures passed
    /// in `#[events]` attribute on a `#[component]` struct. This type is
    /// named by concatenating component name with `Events`.
    type Events;
    /// The state type of a Component.
    ///
    /// ## Internals
    ///
    /// The state type is generated by copying all the state fields from a
    /// `#[component]` struct. This type is named by concatenating component
    /// name with `State`.
    type State: Default;

    /// Creates a new component with the props, events and state passed to it.
    ///
    /// ## Internals
    ///
    /// It initializes the component with props, events passed
    /// to it. The props are used as is whereas the events passed are converted
    /// to `Self::Events` types from the `Self::Events::Other` type.
    ///
    /// It also creates a `Default::default()` state along with wiring up
    /// change notifying mechanism into `status`.
    fn init(props: Self::Props, events: Self::Events, status: Shared<Status<Self::State>>) -> Self;

    /// Updates the component with newer props and returns older props (if
    /// changed).
    ///
    /// ## Internals
    ///
    /// When updating the component with newer props, it compares each prop if
    /// they changed. Also, it updates the events blindly as their is not point
    /// in comparing closures.
    fn update(&mut self, props: Self::Props, events: Self::Events) -> Option<Self::Props>;

    /// Updates the state fields if the status is mutated.
    fn refresh_state(&mut self);

    /// Finds whether the component status has been altered. If altered, resets
    /// it to an undirtied state.
    ///
    /// ## Internals
    ///
    /// Delegates the operation to the `status` field.
    fn take_state_dirty(&self) -> bool;

    /// Finds whether the component has been updated with newer props. If
    /// updated, resets it to undirtied state.
    ///
    /// ## Internals
    ///
    /// Delegates the operation to the `status` field.
    fn take_props_dirty(&self) -> bool;

    /// Mutates the state of the component by executing the closure which
    /// accepts the current state.
    ///
    /// # Example
    /// ```ignore,compile_fail
    /// self.set_state(|state| {
    ///     state.disabled = !state.disabled;
    ///     state.count += 1;
    /// })
    /// ```
    ///
    /// ## Internals
    ///
    /// It mutates the state in the `status` field and checks whether it
    /// differs from the state fields of the component. If they are different
    /// it then marks the state as dirty.
    fn set_state<F>(&self, mutator: F)
    where
        F: FnMut(&mut Self::State);
}

/// Stores the state of the component along with the flags to identify whether
/// the props and state are dirty. Also provides a mechanism to notify the app
/// of state changes.
pub struct Status<T> {
    state: T,
    state_dirty: bool,
    props_dirty: bool,
    rx_sender: MessageSender,
}

impl<T> Status<T> {
    /// Creates a new status with a given state and message sender.
    pub(crate) fn new(state: T, rx_sender: MessageSender) -> Status<T> {
        Status {
            state,
            state_dirty: false,
            props_dirty: false,
            rx_sender,
        }
    }

    /// Marks state as dirty.
    pub fn mark_state_dirty(&mut self) {
        self.state_dirty = true;
    }

    /// Gets and resets `state_dirty` flag.
    pub fn take_state_dirty(&mut self) -> bool {
        if self.state_dirty {
            self.state_dirty = false;
            true
        } else {
            false
        }
    }

    /// Marks props as dirty.
    pub fn mark_props_dirty(&mut self) {
        self.props_dirty = true;
    }

    /// Gets and resets `props_dirty` flag.
    pub fn take_props_dirty(&mut self) -> bool {
        if self.props_dirty {
            self.props_dirty = false;
            true
        } else {
            false
        }
    }

    /// Gets the state immutably.
    pub fn state_as_ref(&self) -> &T {
        &self.state
    }

    /// Gets the state mutably.
    pub fn state_as_mut(&mut self) -> &mut T {
        &mut self.state
    }

    /// Sends a request to the App to react to the state changes.
    pub fn do_react(&self) {
        self.rx_sender.do_react();
    }
}

/// The lifecycle of a stateful component.
///
/// When you do not require these lifecycle hooks, you may implement them with
/// an auto derive `#[derive(Lifecycle)]` on the component struct.
pub trait Lifecycle: Component {
    /// Invoked when the component is first created.
    fn created(&self) {}

    /// Invoked when the component props are updated.
    #[allow(unused_variables)]
    fn updated(&self, old_props: Self::Props) {}

    /// Invoked when the component is mounted onto the DOM tree.
    fn mounted(&self) {}

    /// Invoked when the component is removed from the DOM tree.
    fn destroyed(&self) {}
}

/// Trait to render a view for the component.
pub trait Render: Lifecycle + Sized {
    /// Render a markup for the component by using the html! macro.
    fn render(&self) -> Markup<Self>;
}

/// Trait to convert from a event props to a events type.
///
/// Used to convert a (render) contextual events type to a wrapped one.
/// i.e. `EventProps<RCTX>` to `Events`.
pub trait FromEventProps<RCTX: Render>: Sized {
    /// A contextual events type.
    type From;

    /// Convert to a context wrapped events type.
    fn from(from: Self::From, render_ctx: Shared<RCTX>) -> Self;
}

/// A void component to be used as a render context for a root component.
/// Simply the parent of the root.
pub type RootParent = ();

impl Component for RootParent {
    type Props = ();
    type Events = ();
    type State = ();

    fn init(_: Self::Props, _: Self::Events, _: Shared<Status<()>>) -> RootParent {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }

    fn update(&mut self, _: Self::Props, _: Self::Events) -> Option<Self::Props> {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }

    fn refresh_state(&mut self) {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }

    fn take_state_dirty(&self) -> bool {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }

    fn take_props_dirty(&self) -> bool {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }

    fn set_state<F>(&self, _: F) {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }
}

impl Lifecycle for RootParent {
    fn created(&self) {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }

    fn updated(&self, _: Self::Props) {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }

    fn mounted(&self) {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }

    fn destroyed(&self) {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }
}

impl Render for RootParent {
    fn render(&self) -> Markup<Self> {
        unreachable!(
            "It is a void component to be used as a render context for a root \
             component. Not to be used as a component itself."
        )
    }
}

impl<RCTX: Render> FromEventProps<RCTX> for () {
    type From = ();

    fn from(from: Self::From, _: Shared<RCTX>) -> Self {
        from
    }
}

#[cfg(test)]
pub fn root_render_ctx() -> Shared<()> {
    use std::{cell::RefCell, rc::Rc};

    Rc::new(RefCell::new(()))
}
