use crate::assets::Assets;

use geng::prelude::*;

#[derive(Clone)]
pub struct Context {
    pub geng: Geng,
    pub assets: Rc<Assets>,
}

impl Context {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        Self { geng, assets }
    }
}
