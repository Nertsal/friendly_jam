mod dispatcher;
mod font;
mod solver;

pub use self::{dispatcher::*, font::Font, solver::*};

use crate::render::Color;

use std::path::PathBuf;

use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sounds: SoundAssets,
    pub atlas: SpritesAtlas,
    pub shaders: ShaderAssets,
    pub palette: Palette,
    pub sprites: SpriteAssets,
    pub dispatcher: DispatcherAssets,
    pub solver: SolverAssets,
    #[load(path = "default.ttf")]
    pub font: Rc<Font>,
}

#[derive(geng::asset::Load)]
pub struct SpriteAssets {}

#[derive(geng::asset::Load, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Palette {
    pub background: Color,
}

#[derive(geng::asset::Load)]
pub struct SoundAssets {
    pub click: Rc<geng::Sound>,
    pub hover: Rc<geng::Sound>,
}

#[derive(geng::asset::Load)]
pub struct ShaderAssets {
    pub masked: Rc<ugli::Program>,
    pub texture_ui: Rc<ugli::Program>,
}

friendly_derive::texture_atlas!(pub SpritesAtlas {
    white,

    button_background,
    play_dispatcher,
    play_solver,
});

#[derive(Clone)]
pub struct PixelTexture {
    pub path: PathBuf,
    pub texture: Rc<ugli::Texture>,
}

impl Deref for PixelTexture {
    type Target = ugli::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl Debug for PixelTexture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PixelTexture")
            .field("path", &self.path)
            .field("texture", &"<texture data>")
            .finish()
    }
}

impl geng::asset::Load for PixelTexture {
    type Options = <ugli::Texture as geng::asset::Load>::Options;

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let path = path.to_owned();
        let texture = ugli::Texture::load(manager, &path, options);
        async move {
            let mut texture = texture.await?;
            texture.set_filter(ugli::Filter::Nearest);
            Ok(Self {
                path,
                texture: Rc::new(texture),
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("png");
}
