use chrono::{DateTime, FixedOffset};
use embedded_graphics::{
    geometry::Size,
    image::ImageRaw,
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    pixelcolor::{Rgb565, RgbColor},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder},
    text::{Alignment, Baseline, TextStyle, TextStyleBuilder},
};

use crate::{
    wifi::{RequestData, RequestType, REQUEST_TYPE},
    Button, Display,
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
    image: ImageRaw::new(include_bytes!("../assets/numbers.raw"), 96),
    glyph_mapping: &StrGlyphMapping::new("0123456789. ", 0),
    character_size: Size::new(8, 10),
    character_spacing: 0,
    baseline: 0,
    // TODO: double check this
    underline: DecorationDimensions::default_underline(6),
    strikethrough: DecorationDimensions::default_strikethrough(3),
};

// TODO: add colours here

pub const NUMBER_CHAR: MonoTextStyle<Rgb565> =
    MonoTextStyle::new(&NUMBER_FONT, Rgb565::new(1, 44, 23));

pub const STAT_ONE_CHAR: MonoTextStyle<Rgb565> =
    MonoTextStyle::new(&NUMBER_FONT, Rgb565::new(31, 23, 0));
// stat two is just number char
pub const STAT_THREE_CHAR: MonoTextStyle<Rgb565> =
    MonoTextStyle::new(&NUMBER_FONT, Rgb565::new(25, 27, 28));

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

pub const BACKGROUND: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .fill_color(Rgb565::new(31, 59, 26))
    .build();

pub enum Screens {
    Home,
    Session,
}

impl<'a> Screens {
    pub async fn init(&self) {
        match self {
            Screens::Home => {
                *(REQUEST_TYPE.lock().await) = RequestType::Stats;
            }
            Screens::Session => {
                *(REQUEST_TYPE.lock().await) = RequestType::Session;
            }
        }
    }

    pub async fn input(&self, btn: Button, disp: &mut Display<'a>) {
        match self {
            Screens::Home => home::input(btn, disp).await,
            _ => {} //Screens::Session => home::input(btn, disp).await,
        }
    }

    pub async fn update(
        &self,
        disp: &mut Display<'a>,
        data: RequestData,
        old_count: u16,
        now: DateTime<FixedOffset>,
    ) {
        match self {
            Screens::Home => home::update(disp, data, old_count, now).await,
            Screens::Session => session::update(disp, data, now).await,
        }
    }
}

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
        Image::new(&prev_selected.icon(), prev_selected.icon_pos())
            .draw(disp)
            .unwrap();

        if active != selected {
            Image::new(&selected_btn, selected.pos())
                .draw(disp)
                .unwrap();

            Image::new(&selected.icon(), selected.icon_pos())
                .draw(disp)
                .unwrap();
        }

        if selected == active || prev_selected.is_neighbour_of(active) {
            Image::new(
                &Tga::<Rgb565>::from_slice(ACTIVE_BTN).unwrap(),
                active.pos(),
            )
            .draw(disp)
            .unwrap();
            Image::new(&active.icon(), active.icon_pos())
                .draw(disp)
                .unwrap();
        }
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
    use core::cmp::max;
    use core::{f32::consts::PI, sync::atomic::AtomicBool};

    use chrono::{DateTime, FixedOffset, TimeZone};
    use core::fmt::Write;
    use core::sync::atomic::Ordering;
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
    use log::{error, info};
    use micromath::F32Ext;
    use tinytga::Tga;

    use super::{
        BACKGROUND, CENTERED_TEXT, NUMBER_CHAR, PROGRESS_BG, PROGRESS_BLUE, PROGRESS_ORANGE,
        STAT_ONE_CHAR, STAT_THREE_CHAR,
    };
    use crate::gui::{BLACK_CHAR, NORMAL_TEXT, PICO_FONT};
    use crate::wifi::{RequestData, RUN};
    use crate::{
        check, format, wifi, Button, Display, ARCADE_LOGO, END_DATE, PROGRESS_SELECTED,
        STATS_SELECTED, TICKETS, TICKET_GOAL, TICKET_LARGE, TICKET_OFFSET, TICKET_SMALL,
    };

    static SELECTED: AtomicBool = AtomicBool::new(true);

    pub async fn input(btn: Button, disp: &mut Display<'_>) {
        match btn {
            Button::Up => {
                if !SELECTED.load(Ordering::Relaxed) {
                    SELECTED.store(true, Ordering::Relaxed);

                    Rectangle::new(Point::new(0, 14), Size::new(146, 114))
                        .into_styled(BACKGROUND)
                        .draw(disp)
                        .unwrap();

                    let img: Tga<Rgb565> = Tga::from_slice(PROGRESS_SELECTED).unwrap();
                    Image::new(&img, Point::new(146, 47)).draw(disp).unwrap();
                    RUN.signal(true);
                }
            }
            Button::Down => {
                if SELECTED.load(Ordering::Relaxed) {
                    SELECTED.store(false, Ordering::Relaxed);

                    Rectangle::new(Point::new(0, 14), Size::new(146, 114))
                        .into_styled(BACKGROUND)
                        .draw(disp)
                        .unwrap();

                    let img: Tga<Rgb565> = Tga::from_slice(STATS_SELECTED).unwrap();
                    Image::new(&img, Point::new(146, 47)).draw(disp).unwrap();
                    RUN.signal(true);
                }
            }
            _ => (),
        }
    }

    pub async fn update(
        disp: &mut Display<'_>,
        data: RequestData,
        old_count: u16,
        now: DateTime<FixedOffset>,
    ) {
        let tickets;
        match data {
            RequestData::Stats(ticket_count) => {
                tickets = ticket_count;
            }
            _ => {
                error!("[GUI] [Home] Recieved incorrect data!");
                return;
            }
        }
        match SELECTED.load(Ordering::Relaxed) {
            false => {
                info!("[GUI] Updating stats...");
                update_stats(disp, tickets, now).await;
            }
            true => {
                info!("[GUI] Updating progress bar...");
                update_progress(disp, tickets, old_count, now).await;
            }
        }
    }

    async fn update_progress(
        disp: &mut Display<'_>,
        ticket_count: u16,
        old_count: u16,
        now: DateTime<FixedOffset>,
    ) {
        let logo: Tga<Rgb565> = Tga::from_slice(ARCADE_LOGO).unwrap();
        Image::new(&logo, Point::new(30, 98)).draw(disp).unwrap();

        let img: Tga<Rgb565> = Tga::from_slice(PROGRESS_SELECTED).unwrap();
        Image::new(&img, Point::new(146, 47)).draw(disp).unwrap();

        let mut count = String::<4>::new();
        write!(count, "{} ", ticket_count - TICKET_OFFSET).unwrap();
        info!("{:?}", count);

        Text::with_text_style(&count, Point::new(80, 35), NUMBER_CHAR, CENTERED_TEXT)
            .draw(disp)
            .unwrap();

        let img = match Tga::from_slice(TICKET_LARGE) {
            Ok(img) => img,
            Err(err) => {
                info!("ticket large errored with {:?}", err);
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

        let per = (ticket_count - TICKET_OFFSET) as f32 / TICKET_GOAL as f32;
        info!("Found per");
        Timer::after_nanos(200000).await;

        // TODO: increase max ticket count to 4 digits?
        // TODO: max out percentage to 100%
        let complete = format!(11, "{}% there!", (per * 100.).round());

        info!("got complete");
        Timer::after_nanos(200000).await;

        let ideal = format!(
            28,
            "Should be {}% ({}  ) done!",
            (ideal_percent * 100.).round(),
            (ideal_percent * TICKET_GOAL as f32).round()
        );

        info!("got ideal");
        Timer::after_nanos(200000).await;

        info!(
            "info is {:?}",
            max(
                TICKET_GOAL as i32 - ticket_count as i32 + TICKET_OFFSET as i32,
                0
            ),
            //((1. - per) * 100.).round(),
            // TODO: fix potention overflow
            // its in more places than this
            //TICKET_GOAL,
            //ticket_count,
            //TICKET_OFFSET,
            //TICKET_GOAL - ticket_count + TICKET_OFFSET
        );
        Timer::after_nanos(200000).await;

        let left = format!(
            32,
            "{}% left ({}  )!",
            ((1. - per) * 100.).round() as u16,
            max(
                TICKET_GOAL as i16 - ticket_count as i16 + TICKET_OFFSET as i16,
                0
            ),
        );

        info!("got left");
        Timer::after_nanos(200000).await;

        Text::new(&complete, Point::new(28, 62), BLACK_CHAR)
            .draw(disp)
            .unwrap();

        Text::new(&ideal, Point::new(28, 71), BLACK_CHAR)
            .draw(disp)
            .unwrap();

        Text::new(&left, Point::new(28, 80), BLACK_CHAR)
            .draw(disp)
            .unwrap();

        let img = match Tga::from_slice(TICKET_SMALL) {
            Ok(img) => img,
            Err(err) => {
                info!("ticket small errored with {:?}", err);
                return;
            }
        };

        Image::new(&img, Point::new(28 + ((ideal.len() - 9) * 4) as i32, 70))
            .draw(disp)
            .unwrap();
        Image::new(&img, Point::new(28 + ((left.len() - 4) * 4) as i32, 79))
            .draw(disp)
            .unwrap();

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
            } else if ideal_percent < change + prev {
                RoundedRectangle::with_equal_corners(
                    Rectangle::new(
                        Point::new(0, 0),
                        Size::new((120. * ideal_percent) as u32, 6),
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
    }

    async fn update_stats(disp: &mut Display<'_>, ticket_count: u16, now: DateTime<FixedOffset>) {
        macro_rules! round_format {
            ($num:expr) => {{
                let num = $num;
                let rounded = (num * 10.0).round() / 10.0;
                if rounded.fract() == 0.0 {
                    format!(3, "{}", rounded)
                } else {
                    format!(3, "{:.1}", rounded)
                }
            }};
        }

        let end = FixedOffset::east_opt(14400)
            .unwrap()
            .timestamp_opt(1725163199, 0)
            .unwrap();
        let start = FixedOffset::east_opt(14400)
            .unwrap()
            .timestamp_opt(1718668800, 0)
            .unwrap();

        let hrs = round_format!(
            (ticket_count - TICKET_OFFSET) as f32 / ((now - start).num_days() as f32 + 1.)
        );

        Text::new(&hrs, Point::new(23, 29), STAT_ONE_CHAR)
            .draw(disp)
            .unwrap();
        Text::new(
            "hrs/day on average.",
            Point::new(26 + (hrs.len() * 8) as i32, 31),
            BLACK_CHAR,
        )
        .draw(disp)
        .unwrap();

        let ideal = round_format!(TICKET_GOAL as f32 / (end - start).num_days() as f32);

        Text::new(&ideal, Point::new(23, 45), NUMBER_CHAR)
            .draw(disp)
            .unwrap();

        Text::new(
            "ideal daily tickets.",
            Point::new(26 + (ideal.len() * 8) as i32, 47),
            BLACK_CHAR,
        )
        .draw(disp)
        .unwrap();

        let days_left = format!(2, "{}", (end - now).num_days() - 1);

        Text::new(&days_left, Point::new(23, 61), STAT_THREE_CHAR)
            .draw(disp)
            .unwrap();
        Text::new(
            "days left.",
            Point::new(26 + (days_left.len() * 8) as i32, 63),
            BLACK_CHAR,
        )
        .draw(disp)
        .unwrap();

        let on_track = round_format!(
            (TICKET_GOAL - ticket_count + TICKET_OFFSET) as f32 / (end - now).num_days() as f32
        );

        Text::new(&on_track, Point::new(23, 77), STAT_ONE_CHAR)
            .draw(disp)
            .unwrap();
        Text::new(
            "hrs/day to get on track.",
            Point::new(26 + (on_track.len() * 8) as i32, 79),
            BLACK_CHAR,
        )
        .draw(disp)
        .unwrap();
    }
}

pub mod session {
    use chrono::{DateTime, FixedOffset};
    use log::info;

    use crate::{wifi::RequestData, Display};

    pub async fn update(disp: &mut Display<'_>, data: RequestData, now: DateTime<FixedOffset>) {
        info!("got data from session {:?}", data);
    }
}
