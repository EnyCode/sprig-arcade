pub mod nav {
    use embedded_graphics::{geometry::Point, image::Image, pixelcolor::Rgb565, Drawable};
    use tinytga::Tga;

    use crate::{Display, NavButton, ACTIVE_BTN, BTN, SELECTED_BTN};

    pub fn update_selected(
        selected: &NavButton,
        prev_selected: &NavButton,
        active: &NavButton,
        disp: &mut Display,
    ) {
        let btn: Tga<Rgb565> = Tga::from_slice(if prev_selected == active {
            ACTIVE_BTN
        } else {
            BTN
        })
        .unwrap();
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

    pub fn update_active(active: &NavButton, prev_active: &NavButton, disp: &mut Display) {
        let btn: Tga<Rgb565> = Tga::from_slice(BTN).unwrap();
        let active_btn: Tga<Rgb565> = Tga::from_slice(ACTIVE_BTN).unwrap();

        Image::new(&btn, prev_active.pos()).draw(disp).unwrap();

        Image::new(&active_btn, active.pos()).draw(disp).unwrap();

        Image::new(&active.icon(), active.icon_pos())
            .draw(disp)
            .unwrap();
        Image::new(&prev_active.icon(), prev_active.icon_pos())
            .draw(disp)
            .unwrap();
    }
}

mod home {
    use crate::Display;

    pub fn draw(disp: &mut Display) {}
}
