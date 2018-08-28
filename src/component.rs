use vdom::KeyedVNodes;
use Shared;

/// Trait to define a Component. You do not need to implement this trait.
/// Use the auto derive provided as `#[derive(Component)]`.
///
/// A concrete implementation of a component must have two forms of state.
/// One, the read only state which is accessed by the user. Another, a
/// meta-state which stores the mutable state as well as other helpful values
/// to notify for component re-render.
///
/// Note: The user should not worry about the implementation detail of the
/// actual component. Use the auto derive on the component.
pub trait Component: 'static {
    /// The prop type of a Component.
    type Props;
    /// The event type of a Component.
    type Events;
    /// The state type of a Component.
    type State: Default;

    /// Initializes a component with props as well as a meta-state called status
    fn init<RCTX: Render>(
        props: Self::Props,
        events: Self::Events,
        status: Shared<Status<Self::State>>,
        render_ctx: Shared<RCTX>,
    ) -> Self;

    /// Updated the component with newer props and return older props (if changed). Also, set
    /// the component as dirty (if props changed), so that the component is re-rendered.
    fn update<RCTX: Render>(
        &mut self,
        props: Self::Props,
        events: Self::Events,
        render_ctx: Shared<RCTX>,
    ) -> Option<Self::Props>;

    /// Update the read only state from the mutated status and return true if it has been updated.
    fn refresh_state(&mut self) -> bool;

    /// To find whether the component status has been altered. If altered, reset
    /// it to undirtied state.
    fn is_state_dirty(&mut self) -> bool;

    /// To find whether the component has been updated with newer props. If a newer
    /// props, reset it to undirtied state.
    fn is_props_dirty(&mut self) -> bool;
}

/// Stores the metadata related to the state along with the state.
pub struct Status<T> {
    state: T,
    state_dirty: bool,
    props_dirty: bool,
}

impl<T> Status<T> {
    /// Initialize the status with a given state.
    pub fn new(state: T) -> Status<T> {
        Status {
            state,
            state_dirty: false,
            props_dirty: false,
        }
    }

    /// Get and reset `state_dirty` flag.
    pub fn is_state_dirty(&mut self) -> bool {
        if self.state_dirty {
            self.state_dirty = false;
            true
        } else {
            false
        }
    }

    /// Get and reset `props_dirty` flag.
    pub fn is_props_dirty(&mut self) -> bool {
        if self.props_dirty {
            self.props_dirty = false;
            true
        } else {
            false
        }
    }

    /// Get the state immutably.
    pub fn state_as_ref(&self) -> &T {
        &self.state
    }

    /// Get the state mutably.
    pub fn state_as_mut(&mut self) -> &mut T {
        &mut self.state
    }
}

/// The lifecycle of a stateful component. Implement only the appropriate
/// hooks as needed.
///
/// For stateless component, mark the component with `#[stateless]`.
pub trait Lifecycle: Component {
    /// Invoked when the component is first created
    fn created(&self) {}

    /// Invoked when the component props are updated
    #[allow(unused_variables)]
    fn updated(&self, old_props: Self::Props) {}

    /// Invoked when the component is mounted onto the DOM tree
    fn mounted(&self) {}

    /// Invoked when the component is removed from the DOM tree
    fn destroyed(&self) {}
}

/// Trait to render a view for the component.
pub trait Render: Lifecycle
where
    Self: Sized,
{
    #[allow(missing_docs)]
    fn render(&self) -> KeyedVNodes<Self>;
}