use geng_utils::conversions::Vec2RealConversions;

use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Shape {
    Circle { radius: FCoord },
    Rectangle { width: FCoord, height: FCoord },
}

impl Shape {
    pub fn circle(radius: FCoord) -> Self {
        Self::Circle { radius }
    }

    pub fn rectangle(size: vec2<FCoord>) -> Self {
        Self::Rectangle {
            width: size.x,
            height: size.y,
        }
    }

    pub fn to_parry(self) -> Box<dyn parry2d::shape::Shape> {
        match self {
            Shape::Circle { radius } => Box::new(parry2d::shape::Ball::new(radius.as_f32())),
            Shape::Rectangle { width, height } => {
                if width == R32::ZERO || height == R32::ZERO {
                    return Box::new(parry2d::shape::Ball::new(0.0));
                }
                let aabb = Aabb2::ZERO.extend_symmetric(vec2(width, height).as_f32() / 2.0);
                let points = aabb.corners().map(|p| {
                    let vec2(x, y) = p;
                    parry2d::math::Point::new(x, y)
                });
                match parry2d::shape::ConvexPolygon::from_convex_hull(&points) {
                    Some(shape) => Box::new(shape),
                    None => Box::new(parry2d::shape::Ball::new(0.0)),
                }
            }
        }
    }

    pub fn scaled(self, scale: FCoord) -> Self {
        match self {
            Shape::Circle { radius } => Shape::Circle {
                radius: radius * scale,
            },
            Shape::Rectangle { width, height } => Shape::Rectangle {
                width: width * scale,
                height: height * scale,
            },
        }
    }
}
