use image::{DynamicImage, ImageFormat};
use uuid::Uuid;

use crate::minecraft::PlayerTexture;

const STEVE_BYTES: &'static [u8] = include_bytes!("steve.png");
const ALEX_BYTES: &'static [u8] = include_bytes!("alex.png");

#[derive(Copy, Clone, Debug)]
pub struct Format {
    pub head: CuboidTex,
    pub hat: CuboidTex,
    pub body: CuboidTex,
    pub jacket: Option<CuboidTex>,
    pub right_leg: CuboidTex,
    pub right_pants: Option<CuboidTex>,
    pub left_leg: CuboidTex,
    pub left_pants: Option<CuboidTex>,
    pub right_arm: CuboidTex,
    pub right_sleeves: Option<CuboidTex>,
    pub left_arm: CuboidTex,
    pub left_sleeves: Option<CuboidTex>,
}

impl Format {
    pub const WIDE_ARMS: Format = Format {
        head: CuboidTex::new((0, 0), (8, 8, 8)),
        hat: CuboidTex::new((32, 0), (8, 8, 8)),

        body: CuboidTex::new((16, 16), (8, 12, 4)),
        jacket: Some(CuboidTex::new((16, 32), (8, 12, 4))),

        right_leg: CuboidTex::new((0, 16), (4, 12, 4)),
        right_pants: Some(CuboidTex::new((0, 32), (4, 12, 4))),

        left_leg: CuboidTex::new((16, 48), (4, 12, 4)),
        left_pants: Some(CuboidTex::new((0, 48), (4, 12, 4))),

        right_arm: CuboidTex::new((40, 16), (4, 12, 4)),
        right_sleeves: Some(CuboidTex::new((40, 32), (4, 12, 4))),

        left_arm: CuboidTex::new((32, 48), (4, 12, 4)),
        left_sleeves: Some(CuboidTex::new((48, 48), (4, 12, 4))),
    };

    pub const SLIM_ARMS: Format = Format {
        head: CuboidTex::new((0, 0), (8, 8, 8)),
        hat: CuboidTex::new((32, 0), (8, 8, 8)),

        body: CuboidTex::new((16, 16), (8, 12, 4)),
        jacket: Some(CuboidTex::new((16, 32), (8, 12, 4))),

        right_leg: CuboidTex::new((0, 16), (4, 12, 4)),
        right_pants: Some(CuboidTex::new((0, 32), (4, 12, 4))),

        left_leg: CuboidTex::new((16, 48), (4, 12, 4)),
        left_pants: Some(CuboidTex::new((0, 48), (4, 12, 4))),

        right_arm: CuboidTex::new((40, 16), (3, 12, 4)),
        right_sleeves: Some(CuboidTex::new((40, 32), (3, 12, 4))),

        left_arm: CuboidTex::new((32, 48), (3, 12, 4)),
        left_sleeves: Some(CuboidTex::new((48, 48), (3, 12, 4))),
    };

    // TODO: left arm and leg mirrored?
    pub const LEGACY: Format = Format {
        head: CuboidTex::new((0, 0), (8, 8, 8)),
        hat: CuboidTex::new((32, 0), (8, 8, 8)),

        body: CuboidTex::new((16, 16), (8, 12, 4)),
        jacket: None,

        right_leg: CuboidTex::new((0, 16), (4, 12, 4)),
        right_pants: None,

        left_leg: CuboidTex::new((0, 16), (4, 12, 4)),
        left_pants: None,

        right_arm: CuboidTex::new((40, 16), (4, 12, 4)),
        right_sleeves: None,

        left_arm: CuboidTex::new((40, 16), (4, 12, 4)),
        left_sleeves: None,
    };
}

#[derive(Copy, Clone, Debug)]
pub struct CuboidTex {
    pub front: TexRegion,
    pub back: TexRegion,
    pub top: TexRegion,
    pub bottom: TexRegion,
    pub left: TexRegion,
    pub right: TexRegion,
}

impl CuboidTex {
    pub const fn new(origin: (u32, u32), size: (u32, u32, u32)) -> CuboidTex {
        CuboidTex {
            front: TexRegion::new(
                (origin.0 + size.2, origin.1 + size.2),
                (size.0, size.1),
            ),
            back: TexRegion::new(
                (origin.0 + size.2 + size.0, origin.1 + size.2),
                (size.0, size.1),
            ),
            top: TexRegion::new(
                (origin.0 + size.2, origin.1),
                (size.0, size.2),
            ),
            bottom: TexRegion::new(
                (origin.0 + size.2 + size.0, origin.1),
                (size.0, size.2),
            ),
            left: TexRegion::new(
                (origin.0 + size.2 + 2 * size.0, origin.1 + size.2),
                (size.2, size.1),
            ),
            right: TexRegion::new(
                (origin.0, origin.1 + size.2),
                (size.2, size.1),
            ),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TexRegion {
    pub origin: (u32, u32),
    pub size: (u32, u32),
}

impl TexRegion {
    pub const fn new(origin: (u32, u32), size: (u32, u32)) -> TexRegion {
        TexRegion { origin, size }
    }
}

#[derive(Clone)]
pub struct Skin {
    pub image: image::RgbaImage,
    pub format: Format,
}

impl Skin {
    pub fn from(texture: PlayerTexture) -> Option<Skin> {
        let model = texture.metadata.get("model");
        let model = match model.map(|s| s.as_str()) {
            Some("slim") => Model::Slim,
            _ => Model::Wide,
        };

        let format = match (model, texture.image.dimensions()) {
            (Model::Wide, (64, 32)) => Format::LEGACY,
            (Model::Wide, (64, 64)) => Format::WIDE_ARMS,
            (Model::Slim, (64, 64)) => Format::SLIM_ARMS,
            _ => return None,
        };

        Some(Skin {
            image: texture.image,
            format,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Model {
    Wide,
    Slim,
}

#[derive(Debug, Copy, Clone)]
pub enum DefaultSkin {
    Steve,
    Alex,
}

impl DefaultSkin {
    #[inline]
    pub fn as_skin(&self) -> &Skin {
        use lazy_static::lazy_static;

        lazy_static! {
            static ref STEVE: Skin = load_default_skin(STEVE_BYTES, Format::WIDE_ARMS);
            static ref ALEX: Skin = load_default_skin(ALEX_BYTES, Format::SLIM_ARMS);
        }

        match self {
            DefaultSkin::Steve => &STEVE,
            DefaultSkin::Alex => &ALEX,
        }
    }
}

fn load_default_skin(bytes: &'static [u8], format: Format) -> Skin {
    let cursor = std::io::Cursor::new(bytes);
    match image::io::Reader::with_format(cursor, ImageFormat::Png).decode() {
        Ok(DynamicImage::ImageRgba8(image)) => Skin { image, format },
        _ => panic!("malformed default skins"),
    }
}

impl From<Uuid> for DefaultSkin {
    fn from(uuid: Uuid) -> Self {
        let uuid = uuid.as_u128();

        // UUID.hashCode()
        let msb = ((uuid >> 64) & u64::MAX as u128) as i64;
        let lsb = (uuid & u64::MAX as u128) as i64;
        let hilo = msb ^ lsb;
        let hash = ((hilo >> 32) as i32) ^ (hilo as i32);

        if (hash & 1) == 0 {
            DefaultSkin::Steve
        } else {
            DefaultSkin::Alex
        }
    }
}
