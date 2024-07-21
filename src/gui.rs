use embedded_graphics::{
    geometry::Size,
    image::ImageRaw,
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    pixelcolor::{Rgb565, RgbColor},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder},
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

const NUMBER_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("../assets/numbers.raw"), 80),
    glyph_mapping: &StrGlyphMapping::new("0123456789", 0),
    character_size: Size::new(8, 10),
    character_spacing: 0,
    baseline: 0,
    // TODO: double check this
    underline: DecorationDimensions::default_underline(6),
    strikethrough: DecorationDimensions::default_strikethrough(3),
};

pub const NUMBER_CHAR: MonoTextStyle<Rgb565> =
    MonoTextStyle::new(&NUMBER_FONT, Rgb565::new(1, 44, 23));
pub const BLACK_CHAR: MonoTextStyle<Rgb565> = MonoTextStyle::new(&PICO_FONT, Rgb565::BLACK);
pub const NORMAL_TEXT: TextStyle = TextStyleBuilder::new()
    .baseline(Baseline::Alphabetic)
    .alignment(Alignment::Left)
    .build();
pub const CENTERED_TEXT: TextStyle = TextStyleBuilder::new()
    .baseline(Baseline::Middle)
    .alignment(Alignment::Center)
    .build();

pub const PROGRESS_BG: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .fill_color(Rgb565::new(30, 57, 24))
    .build();

pub const PROGRESS_BLUE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .fill_color(Rgb565::new(1, 44, 23))
    .build();

pub const PROGRESS_ORANGE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .fill_color(Rgb565::new(31, 23, 0))
    .build();

pub mod nav {
    use embedded_graphics::{image::Image, pixelcolor::Rgb565, Drawable};
    use log::info;
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

pub mod home {
    use core::{f32::consts::PI, sync::atomic::AtomicBool};

    use chrono::{DateTime, FixedOffset, TimeZone};
    use core::fmt::Write;
    use embassy_time::Timer;
    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::{
        geometry::{Point, Size},
        image::Image,
        pixelcolor::Rgb565,
        primitives::{Primitive, Rectangle, RoundedRectangle},
        text::Text,
        Drawable,
    };
    use embedded_graphics_framebuf::FrameBuf;
    use heapless::String;
    use log::info;
    use micromath::F32Ext;
    use tinytga::Tga;

    use super::{CENTERED_TEXT, NUMBER_CHAR, PROGRESS_BG, PROGRESS_BLUE, PROGRESS_ORANGE};
    use crate::gui::{BLACK_CHAR, NORMAL_TEXT, PICO_FONT};
    use crate::{Display, END_DATE, TICKET_GOAL, TICKET_LARGE, TICKET_OFFSET};

    pub async fn init(disp: &mut Display<'_>) {
        // NOTE FOR SELF: by this point the display has been cleared
        // TODO: move arcade logo drawing to here
    }

    pub async fn update_progress(
        disp: &mut Display<'_>,
        ticket_count: u16,
        old_count: u16,
        now: DateTime<FixedOffset>,
    ) {
        info!("[GUI] updating gui");
        Timer::after_nanos(200000).await;

        let mut count = String::<4>::new();
        write!(count, "{} ", ticket_count - TICKET_OFFSET).unwrap();
        info!("{:?}", count);

        Text::with_text_style(&count, Point::new(80, 35), NUMBER_CHAR, CENTERED_TEXT)
            .draw(disp)
            .unwrap();

        let img = match Tga::from_slice(TICKET_LARGE) {
            Ok(img) => img,
            Err(err) => {
                info!("errored with {:?}", err);
                return;
            }
        };

        Image::new(&img, Point::new(80 + (3 * (count.len() - 1)) as i32, 28))
            .draw(disp)
            .unwrap();

        info!("checking days left");
        Timer::after_nanos(200000).await;
        let end = FixedOffset::east_opt(14400)
            .unwrap()
            .timestamp_opt(1725163199, 0)
            .unwrap();
        let start = FixedOffset::east_opt(14400)
            .unwrap()
            .timestamp_opt(1718668800, 0)
            .unwrap();

        let days_left = end - now;
        let passed_days = now - start;
        info!(
            "We are {:?} days into Arcade out of {:?} days",
            passed_days.num_days(),
            (end - start).num_days()
        );
        let ideal_percent =
            (passed_days.num_days() as f32 + 1.0) / ((end - start).num_days() as f32 - 1.0);
        info!(
            "The ideal percentage is {:?} with {:?} tickets",
            ideal_percent,
            ideal_percent * TICKET_GOAL as f32
        );
        //let ideal_tickets = TICKET_GOAL as f32 / days_left.num_days() as f32;

        let old = old_count
            - if TICKET_OFFSET > old_count {
                old_count
            } else {
                TICKET_OFFSET
            };

        // 1 second long animation
        // TODO: expected progress
        let prev = old as f32 / TICKET_GOAL as f32;
        let change = (ticket_count - old) as f32 / TICKET_GOAL as f32;

        static DRAWN: AtomicBool = AtomicBool::new(false);
        let mut data = [Rgb565::new(31, 60, 27); 120 * 6];
        let mut fbuf = FrameBuf::new(&mut data, 120, 6);

        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(0, 0), Size::new(120, 6)),
            Size::new(2, 2),
        )
        .into_styled(PROGRESS_BG)
        .draw(&mut fbuf)
        .unwrap();

        for i in 0..30 {
            let mul = -((PI * (i as f32 / 30.)).cos() - 1.) / 2.;

            if !DRAWN.load(core::sync::atomic::Ordering::Relaxed) && ideal_percent >= change + prev
            {
                RoundedRectangle::with_equal_corners(
                    Rectangle::new(
                        Point::new(0, 0),
                        Size::new((120. * (ideal_percent * mul)) as u32, 6),
                    ),
                    Size::new(2, 2),
                )
                .into_styled(PROGRESS_ORANGE)
                .draw(&mut fbuf)
                .unwrap();
            }

            RoundedRectangle::with_equal_corners(
                Rectangle::new(
                    Point::new(0, 0),
                    Size::new((120. * ((change * mul) + prev)) as u32, 6),
                ),
                Size::new(2, 2),
            )
            .into_styled(PROGRESS_BLUE)
            .draw(&mut fbuf)
            .unwrap();

            if !DRAWN.load(core::sync::atomic::Ordering::Relaxed) && ideal_percent < change + prev {
                RoundedRectangle::with_equal_corners(
                    Rectangle::new(
                        Point::new(0, 0),
                        Size::new((120. * (ideal_percent * mul)) as u32, 6),
                    ),
                    Size::new(2, 2),
                )
                .into_styled(PROGRESS_ORANGE)
                .draw(&mut fbuf)
                .unwrap();
            }

            let area = Rectangle::new(Point::new(20, 53), fbuf.size());

            disp.fill_contiguous(&area, *fbuf.data).unwrap();

            Timer::after_millis(33).await;
        }

        DRAWN.store(true, core::sync::atomic::Ordering::Relaxed);

        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(20, 62), Size::new(6, 6)),
            Size::new(2, 2),
        )
        .into_styled(PROGRESS_BLUE)
        .draw(disp)
        .unwrap();
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(20, 71), Size::new(6, 6)),
            Size::new(2, 2),
        )
        .into_styled(PROGRESS_ORANGE)
        .draw(disp)
        .unwrap();
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(20, 80), Size::new(6, 6)),
            Size::new(2, 2),
        )
        .into_styled(PROGRESS_BG)
        .draw(disp)
        .unwrap();

        Text::new("82% there!", Point::new(28, 62), BLACK_CHAR)
            .draw(disp)
            .unwrap();

        Text::new("Should be 46% (74 ) done!", Point::new(28, 71), BLACK_CHAR)
            .draw(disp)
            .unwrap();

        Text::new("18% left (29 )!", Point::new(28, 80), BLACK_CHAR)
            .draw(disp)
            .unwrap();
    }
}
