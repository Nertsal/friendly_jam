pub mod dispatcher;
pub mod solver;

pub use self::{dispatcher::GameDispatcher, solver::GameSolver};

use crate::context::Context;

use geng::prelude::*;
