pub mod nav {
    use embedded_graphics::{geometry::Point, image::Image, pixelcolor::Rgb565, Drawable};
    use tinytga::Tga;

    use crate::{Display, NavButton, BTN, SELECTED_BTN};

    pub fn update_selected(selected: NavButton, prev_selected: NavButton, disp: &mut Display) {
        let btn: Tga<Rgb565> = Tga::from_slice(BTN).unwrap();
        let selected_btn: Tga<Rgb565> = Tga::from_slice(SELECTED_BTN).unwrap();

        Image::new(&btn, prev_selected.pos()).draw(disp).unwrap();

        Image::new(&selected_btn, selected.pos())
            .draw(disp)
            .unwrap();

        Image::new(&selected.icon(), selected.icon_pos())
            .draw(disp)
            .unwrap();
        Image::new(&prev_selected.icon(), prev_selected.icon_pos())
            .draw(disp)
            .unwrap();
    }
}
