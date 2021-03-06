extern crate radiant_rs;
extern crate radiant_utils as ru;
use radiant_rs::{Display, Renderer, Layer, Sprite, Color, blendmodes};

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Sprites example").build();

    // Create a renderer. It is used to draw to a rendertarget (usually a frame).
    let renderer = Renderer::new(&display).unwrap();

    // Create a sprite from a spritesheet file, extracting frame layout from filename.
    let sprite = Sprite::from_file(&renderer.context(), r"examples/res/sprites/ball_v2_32x32x18.jpg").unwrap();

    // A layer where 320x240 units correspond to the full window (which measures 640x480 pixels, so that one unit = two pixel).
    let layer = Layer::new((320., 240.));

    // Layers have a blendmode setting that defines how their contents will be blended with the background on draw.
    layer.set_blendmode(blendmodes::LIGHTEN);

    ru::renderloop(|frame| {

        // Clear the layer (layers could also be drawn multiple times, e.g. a static UI might not need to be updated each frame)
        layer.clear();

        // Draw three sprites to the layer, multiplied by colors red, green and blue as well as the original sprite (multiplied by white, which is the identity)
        let frame_id = (frame.elapsed_f32 * 30.0) as u32;
        sprite.draw(&layer, frame_id, (160., 120.), Color::WHITE);
        sprite.draw(&layer, frame_id, (130., 100.), Color::RED);
        sprite.draw(&layer, frame_id, (190., 100.), Color::GREEN);
        sprite.draw(&layer, frame_id, (160., 155.), Color::BLUE);

        // draw the layer to the frame after clearing it with solid black.
        display.clear_frame(Color::BLACK);
        renderer.draw_layer(&layer, 0);

        display.swap_frame();
        !display.poll_events().was_closed()
    });
}
