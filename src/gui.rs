use embedded_graphics::{
    geometry::Size,
    image::ImageRaw,
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    pixelcolor::{Rgb565, RgbColor},
    text::{Alignment, Baseline, TextStyle, TextStyleBuilder},
};

const PICO_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("../assets/font.raw"), 128),
    glyph_mapping: &StrGlyphMapping::new(
        "  ! \" # $ % & ' ( ) * + , - . / 0 1 2 3 4 5 6 7 8 9 : ; < = > ? @ A B C D E F G H I J K L M N O P Q R S T U V W X Y Z [ \\ ] ^ _ ` a b c d e f g h i j k l m n o p q r s t u v w x y z { | } ~ \u{80} \u{81}\u{82}\u{83}\u{84}\u{85}\u{86}\u{87}\u{88}\u{89}\u{8A}\u{8B}\u{8C}\u{8D}\u{8E}\u{8F}\u{90}\u{91}\u{92}\u{93}\u{94}\u{95}\u{96}\u{97}\u{98}\u{99}\u{9A}\u{9B}\u{9C}\u{9D}\u{9E}\u{9F}\u{A0}\u{A1}\u{A2}\u{A3}\u{A4}\u{A5}\u{A6}\u{A7}\u{A8}\u{A9}\u{AA}\u{AB}\u{AC}\u{AD}\u{AE}\u{AF}\u{B0}\u{B1}\u{B2}\u{B3}\u{B4}\u{B5}\u{B6}\u{B7}\u{B8}\u{B9}\u{BA}\u{BB}\u{BC}",
        0,
    ),
    character_size: Size::new(4, 6),
    character_spacing: 0,
    baseline: 0,
    // TODO: double check this
    underline: DecorationDimensions::default_underline(6),
    strikethrough: DecorationDimensions::default_strikethrough(3),
};

pub const WHITE_CHAR: MonoTextStyle<Rgb565> = MonoTextStyle::new(&PICO_FONT, Rgb565::WHITE);
pub const GREY_CHAR: MonoTextStyle<Rgb565> =
    MonoTextStyle::new(&PICO_FONT, Rgb565::new(24, 49, 24));
pub const BLACK_CHAR: MonoTextStyle<Rgb565> = MonoTextStyle::new(&PICO_FONT, Rgb565::BLACK);
pub const NORMAL_TEXT: TextStyle = TextStyleBuilder::new()
    .baseline(Baseline::Alphabetic)
    .alignment(Alignment::Left)
    .build();
pub const CENTERED_TEXT: TextStyle = TextStyleBuilder::new()
    .baseline(Baseline::Middle)
    .alignment(Alignment::Center)
    .build();

pub mod nav {
    use embedded_graphics::{image::Image, pixelcolor::Rgb565, Drawable};
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
