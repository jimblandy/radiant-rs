extern crate radiant_rs;
use radiant_rs::*;

pub fn main() {
    let display = Display::builder().dimensions((640, 480)).vsync().title("Color accuracy test").build();
    display.clear_frame(Color::BLACK);

    let renderer = Renderer::new(&display).unwrap();

    let layer = Layer::new((640., 480.));
    layer.set_blendmode(blendmodes::ALPHA);

    let sprite = Sprite::from_file(&renderer.context(), r"examples/res/sprites/test_256x256x1.png").unwrap();
    sprite.draw(&layer, 0, (128., 128.), Color::WHITE);

    let surface = Texture::new(&renderer.context(), 300, 300);
    let surface_mat = math::Mat4::viewport(300., 300.);

    let draw_it = |offset: f32| {
        // render to texture and draw that
        renderer.render_to(&surface, || {
            layer.view_matrix().push().set(surface_mat).translate((offset, offset));
            renderer.draw_layer(&layer, 0);
            layer.view_matrix().pop();
        });
        // directly render to frame
        layer.view_matrix().push().translate((offset, offset));
        renderer.draw_layer(&layer, 0);
        layer.view_matrix().pop();
    };

    // draw to screen and texture
    draw_it(10.);
    draw_it(20.);
    draw_it(30.);

    // draw the texture
    renderer.rect((300., 0., 300., 300.)).blendmode(blendmodes::ALPHA).texture(&surface).draw();

    // show frame and wait for exit
    display.swap_frame();
    utils::renderloop(|_| !display.poll_events().was_closed());
}
