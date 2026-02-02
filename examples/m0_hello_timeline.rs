use anyhow::Result;

use raylib_playground::{
    Clip, Color, ImageObject, Layer, Object, RaylibPreview, Shape, Timeline, Transform, Vec2,
};

fn main() -> Result<()> {
    // 10-second timeline at 30 FPS (timeline time is always seconds).
    let mut timeline = Timeline::new(10.0, 30)?;

    // Background layer: a large rectangle centered on the canvas.
    let mut background = Layer::new("background");
    background.add_clip(Clip::new(
        0.0,
        8.0,
        Object::Shape(Shape::Rect {
            width: 700.0,
            height: 420.0,
            // Note: this clip covers the screen, so it overrides the preview clear color.
            //color: Color::rgb(128, 5, 128),
            color: Color::rgba(128, 4, 128, 255),
        }),
        Transform::default(),
        timeline.duration,
    )?);

    // Mid layer: two shapes that overlap in time to show z-ordering.
    let mut mid = Layer::new("mid");
    mid.add_clip(Clip::new(
        1.0,
        6.5,
        Object::Shape(Shape::Circle {
            radius: 80.0,
            color: Color::rgba_css(235, 101, 80, 0.7),
        }),
        Transform {
            // Graph coords: (0,0) is center; +Y is up.
            pos: Vec2 { x: -140.0, y: 60.0 },
            ..Transform::default()
        },
        timeline.duration,
    )?);
    mid.add_clip(Clip::new(
        2.5,
        8.0,
        Object::Shape(Shape::Rect {
            width: 200.0,
            height: 120.0,
            color: Color::rgb(70, 140, 220),
        }),
        Transform {
            pos: Vec2 { x: 160.0, y: -40.0 },
            rotation: 12.0,
            ..Transform::default()
        },
        timeline.duration,
    )?);

    // Top layer: image clip that appears later.
    let mut top = Layer::new("top");
    top.add_clip(Clip::new(
        3.0,
        8.0,
        Object::Image(ImageObject::new("assets/logo.png")),
        Transform {
            pos: Vec2 { x: 0.0, y: 0.0 },
            scale: Vec2 { x: 2.0, y: 2.0 },
            ..Transform::default()
        },
        timeline.duration,
    )?);

    timeline.add_layer(background);
    timeline.add_layer(mid);
    timeline.add_layer(top);

    // Preview window uses a clear color behind the timeline.
    let preview = RaylibPreview::new(800, 600, Color::rgb(16, 16, 20));
    preview.run(&timeline)
}
