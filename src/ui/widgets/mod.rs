use super::*;

mod button;
mod input;
mod text;

pub use self::{button::*, input::*, text::*};

use std::any::Any;

#[macro_export]
macro_rules! simple_widget_state {
    () => {
        fn state_mut(&mut self) -> &mut WidgetState {
            &mut self.state
        }
    };
    ($path:tt) => {
        fn state_mut(&mut self) -> &mut WidgetState {
            &mut self.$path.state
        }
    };
}

pub trait Widget: WidgetToAny {
    fn state_mut(&mut self) -> &mut WidgetState;
    #[must_use]
    fn draw_top(&self, context: &UiContext) -> Geometry {
        #![allow(unused_variables)]
        Geometry::new()
    }
    #[must_use]
    fn draw(&self, context: &UiContext) -> Geometry;
}

#[doc(hidden)]
pub trait WidgetToAny {
    fn to_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> WidgetToAny for T {
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
