use chargrid::{control_flow::*, core::*};

pub fn app() -> App {
    render(|ctx, fb| {
        let size = ctx.bounding_box.size();
        for y in 0..size.height() {
            for x in 0..size.width() {
                let coord = Coord::new(x as i32, y as i32);
                let r = 255 - ((x * 255) / size.width());
                let g = (x * 510) / size.width();
                let g = if g > 255 { 510 - g } else { g };
                let b = (x * 255) / size.width();
                let mul = 255 - ((y * 255) / size.height());
                let col =
                    Rgba32::new_rgb(r as u8, g as u8, b as u8).normalised_scalar_mul(mul as u8);
                let cell = RenderCell::default()
                    .with_character(' ')
                    .with_background(col)
                    .with_foreground(col);
                fb.set_cell_relative_to_ctx(ctx, coord, 0, cell);
            }
        }
    })
    .press_any_key()
    .map(|()| app::Exit)
}
