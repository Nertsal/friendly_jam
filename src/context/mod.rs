use crate::assets::Assets;

use geng::prelude::*;

#[derive(Clone)]
pub struct Context {
    pub geng: Geng,
    pub assets: Rc<Hot<Assets>>,
    pub music: MusicManager,
}

impl Context {
    pub fn new(geng: Geng, assets: Rc<Hot<Assets>>) -> Self {
        Self {
            geng,
            assets,
            music: MusicManager::new(),
        }
    }
}

#[derive(Clone)]
pub struct MusicManager {
    inner: Rc<RefCell<Music>>,
}

struct Music {
    fx: Option<geng::SoundEffect>,
}

impl MusicManager {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Music { fx: None })),
        }
    }

    pub fn play_music(&self, music: &geng::Sound) {
        let mut inner = self.inner.borrow_mut();
        if let Some(mut fx) = inner.fx.take() {
            fx.fade_out(time::Duration::from_secs_f64(0.5));
        }

        let mut fx = music.play();
        fx.fade_to_volume(0.3, time::Duration::from_secs_f64(0.5));
        inner.fx = Some(fx);
    }

    pub fn fade_temporarily(&self, volume: f32, duration: time::Duration) {
        let mut inner = self.inner.borrow_mut();
        if let Some(fx) = &mut inner.fx {
            fx.set_volume(volume * 0.3);
            fx.fade_to_volume(0.3, duration);
        }
    }
}
