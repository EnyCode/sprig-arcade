use embassy_executor::Spawner;
use embassy_rp::rtc::DateTime;
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
    image: ImageRaw::new(include_bytes!("../assets/numbers.raw"), 136),
    glyph_mapping: &StrGlyphMapping::new("0123456789. :DONE", 0),
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

// TODO: change to not solid black
pub const BLACK_NUMBER_CHAR: MonoTextStyle<Rgb565> =
    MonoTextStyle::new(&NUMBER_FONT, Rgb565::BLACK);

pub const BLACK_CHAR: MonoTextStyle<Rgb565> = MonoTextStyle::new(&PICO_FONT, Rgb565::BLACK);
pub const CENTERED_TEXT: TextStyle = TextStyleBuilder::new()
    .baseline(Baseline::Alphabetic)
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

pub const BLACK_FILL: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
    .fill_color(Rgb565::BLACK)
    .build();

fn days_since_epoch(datetime: &DateTime) -> i32 {
    let is_leap_year =
        datetime.year % 4 == 0 && (datetime.year % 100 != 0 || datetime.year % 400 == 0);
    let mut days = (datetime.year as i32 - 1) * 365;
    days += (datetime.year as i32 - 1) / 4 - (datetime.year as i32 - 1) / 100
        + (datetime.year as i32 - 1) / 400;
    let mut month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if is_leap_year {
        month_days[2] = 29;
    }
    for i in 1..datetime.month {
        days += month_days[i as usize];
    }
    days += datetime.day as i32;
    days
}

fn days_between(date: &DateTime, other: &DateTime) -> i32 {
    let days1 = days_since_epoch(date);
    let days2 = days_since_epoch(other);
    (days1 - days2).abs()
}

pub enum Screens {
    Home,
    Session,
}

impl<'a> Screens {
    pub async fn init(&self, spawner: &Spawner) {
        session::ON_SCREEN.reset();
        match self {
            Screens::Home => {
                *(REQUEST_TYPE.lock().await) = RequestType::Stats;
                home::init().await;
            }
            Screens::Session => {
                *(REQUEST_TYPE.lock().await) = RequestType::Session;
                session::init(spawner).await;
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
        now: DateTime,
    ) {
        match self {
            Screens::Home => home::update(disp, data, old_count, now).await,
            Screens::Session => session::update(disp, data, now).await,
        }
    }
}

pub mod nav {
    use embedded_graphics::{image::Image, pixelcolor::Rgb565, Drawable};
    use tinytga::Tga;

    use crate::{
        util::{ACTIVE_BTN, BTN, SELECTED_BTN},
        Display, NavButton,
    };

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
    use core::fmt::Write;
    use core::sync::atomic::Ordering;
    use core::{f32::consts::PI, sync::atomic::AtomicBool};
    use embassy_rp::rtc::{DateTime, DayOfWeek};
    use embassy_time::Timer;
    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::{
        geometry::{Point, Size},
        image::Image,
        pixelcolor::Rgb565,
        primitives::{Primitive, Rectangle},
        Drawable,
    };
    use embedded_graphics_framebuf::FrameBuf;
    use heapless::String;
    use log::{debug, error, info};
    use micromath::F32Ext;
    use tinytga::Tga;

    use super::{
        BACKGROUND, CENTERED_TEXT, PROGRESS_BG, PROGRESS_BLUE, PROGRESS_ORANGE, STAT_ONE_CHAR,
        STAT_THREE_CHAR,
    };
    use crate::gui::{days_between, NUMBER_CHAR};
    use crate::wifi::{RequestData, RUN};
    use crate::{
        draw_rect, draw_rounded_rect, draw_tga, write_large_text, write_text, UPDATE_INTERVAL,
    };
    use crate::{
        format,
        util::{ARCADE_LOGO, PROGRESS_SELECTED, STATS_SELECTED, TICKET_LARGE, TICKET_SMALL},
        Button, Display, TICKET_GOAL, TICKET_OFFSET,
    };

    static SELECTED: AtomicBool = AtomicBool::new(true);

    pub async fn init() {
        UPDATE_INTERVAL.store(5, core::sync::atomic::Ordering::Relaxed);
    }

    pub async fn input(btn: Button, disp: &mut Display<'_>) {
        match btn {
            Button::Up => {
                if !SELECTED.load(Ordering::Relaxed) {
                    SELECTED.store(true, Ordering::Relaxed);

                    draw_rect!(Point::new(0, 14), Size::new(146, 114), BACKGROUND, disp);
                    draw_tga!(PROGRESS_SELECTED, Point::new(146, 47), disp);

                    RUN.signal(true);
                }
            }
            Button::Down => {
                if SELECTED.load(Ordering::Relaxed) {
                    SELECTED.store(false, Ordering::Relaxed);

                    draw_rect!(Point::new(0, 14), Size::new(146, 114), BACKGROUND, disp);
                    draw_tga!(STATS_SELECTED, Point::new(146, 47), disp);

                    RUN.signal(true);
                }
            }
            _ => (),
        }
    }

    pub async fn update(disp: &mut Display<'_>, data: RequestData, old_count: u16, now: DateTime) {
        let tickets;
        match data {
            RequestData::Stats(ticket_count) => {
                tickets = ticket_count;
            }
            _ => {
                error!("[GUI] Recieved incorrect data!");
                return;
            }
        }
        match SELECTED.load(Ordering::Relaxed) {
            false => {
                debug!("[GUI] Updating stats...");
                update_stats(disp, tickets, now).await;
            }
            true => {
                debug!("[GUI] Updating progress bar...");
                update_progress(disp, tickets, old_count, now).await;
            }
        }
    }

    async fn update_progress(
        disp: &mut Display<'_>,
        ticket_count: u16,
        old_count: u16,
        now: DateTime,
    ) {
        draw_tga!(ARCADE_LOGO, Point::new(30, 98), disp);
        draw_tga!(PROGRESS_SELECTED, Point::new(146, 47), disp);

        let mut count = String::<4>::new();
        write!(count, "{} ", ticket_count - TICKET_OFFSET).unwrap();
        debug!("[GUI] {:?}", count);
        write_text!(
            custom,
            &count,
            Point::new(80, 35),
            NUMBER_CHAR,
            CENTERED_TEXT,
            disp
        );

        draw_tga!(
            TICKET_LARGE,
            Point::new(80 + (3 * (count.len() - 1)) as i32, 28),
            disp
        );

        let end = DateTime {
            year: 2024,
            month: 9,
            day: 31,
            hour: 23,
            minute: 59,
            second: 59,
            day_of_week: DayOfWeek::Saturday,
        };
        let start = DateTime {
            year: 2024,
            month: 6,
            day: 18,
            hour: 0,
            minute: 0,
            second: 0,
            day_of_week: DayOfWeek::Tuesday,
        };

        let passed_days = days_between(&now, &start);
        let ideal_percent = (passed_days as f32 + 1.0) / (days_between(&end, &start) as f32 - 1.0);

        let old = old_count
            - if TICKET_OFFSET > old_count {
                old_count
            } else {
                TICKET_OFFSET
            };
        draw_rounded_rect!(
            Point::new(20, 62),
            Size::new(6, 6),
            Size::new(2, 2),
            PROGRESS_BLUE,
            disp
        );
        draw_rounded_rect!(
            Point::new(20, 71),
            Size::new(6, 6),
            Size::new(2, 2),
            PROGRESS_ORANGE,
            disp
        );
        draw_rounded_rect!(
            Point::new(20, 80),
            Size::new(6, 6),
            Size::new(2, 2),
            PROGRESS_BG,
            disp
        );

        let per = (ticket_count - TICKET_OFFSET) as f32 / TICKET_GOAL as f32;

        // TODO: max out percentage to 100%
        let complete = format!(11, "{}% there!", (per * 100.).round());

        let ideal = format!(
            28,
            "Should be {}% ({}  ) done!",
            (ideal_percent * 100.).round(),
            (ideal_percent * TICKET_GOAL as f32).round()
        );

        let left = format!(
            32,
            "{}% left ({}  )!",
            ((1. - per) * 100.).round() as u16,
            max(
                TICKET_GOAL as i16 - ticket_count as i16 + TICKET_OFFSET as i16,
                0
            ),
        );

        write_text!(&complete, Point::new(28, 62), disp);
        write_text!(&ideal, Point::new(28, 71), disp);
        write_text!(&left, Point::new(28, 80), disp);

        let img = Tga::from_slice(TICKET_SMALL).unwrap();
        draw_tga!(
            tga,
            img,
            Point::new(28 + ((ideal.len() - 9) * 4) as i32, 70),
            disp
        );
        draw_tga!(
            tga,
            img,
            Point::new(28 + ((left.len() - 4) * 4) as i32, 79),
            disp
        );

        let prev = old as f32 / TICKET_GOAL as f32;
        let change = (ticket_count - TICKET_OFFSET - old) as f32 / TICKET_GOAL as f32;

        static DRAWN: AtomicBool = AtomicBool::new(false);
        let mut data = [Rgb565::new(31, 60, 27); 120 * 6];
        let mut fbuf = FrameBuf::new(&mut data, 120, 6);

        draw_rounded_rect!(
            Point::new(0, 0),
            Size::new(120, 6),
            Size::new(2, 2),
            PROGRESS_BG,
            &mut fbuf
        );

        for i in 0..30 {
            let mul = -((PI * (i as f32 / 30.)).cos() - 1.) / 2.;

            if !DRAWN.load(core::sync::atomic::Ordering::Relaxed) && ideal_percent >= change + prev
            {
                draw_rounded_rect!(
                    Point::new(0, 0),
                    Size::new((120. * (ideal_percent * mul)) as u32, 6),
                    Size::new(2, 2),
                    PROGRESS_ORANGE,
                    &mut fbuf
                );
            }

            draw_rounded_rect!(
                Point::new(0, 0),
                Size::new((120. * ((change * mul) + prev)) as u32, 6),
                Size::new(2, 2),
                PROGRESS_BLUE,
                &mut fbuf
            );

            if !DRAWN.load(core::sync::atomic::Ordering::Relaxed) && ideal_percent < change + prev {
                draw_rounded_rect!(
                    Point::new(0, 0),
                    Size::new((120. * (ideal_percent * mul)) as u32, 6),
                    Size::new(2, 2),
                    PROGRESS_ORANGE,
                    &mut fbuf
                );
            } else if ideal_percent < change + prev {
                draw_rounded_rect!(
                    Point::new(0, 0),
                    Size::new((120. * ideal_percent) as u32, 6),
                    Size::new(2, 2),
                    PROGRESS_ORANGE,
                    &mut fbuf
                );
            }

            let area = Rectangle::new(Point::new(20, 53), fbuf.size());

            disp.fill_contiguous(&area, *fbuf.data).unwrap();

            Timer::after_millis(33).await;
        }

        DRAWN.store(true, core::sync::atomic::Ordering::Relaxed);
    }

    async fn update_stats(disp: &mut Display<'_>, ticket_count: u16, now: DateTime) {
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

        let end = DateTime {
            year: 2024,
            month: 9,
            day: 31,
            hour: 23,
            minute: 59,
            second: 59,
            day_of_week: DayOfWeek::Saturday,
        };
        let start = DateTime {
            year: 2024,
            month: 6,
            day: 18,
            hour: 0,
            minute: 0,
            second: 0,
            day_of_week: DayOfWeek::Tuesday,
        };

        let hrs = round_format!(
            (ticket_count - TICKET_OFFSET) as f32 / (days_between(&now, &start) as f32 + 1.)
        );
        write_text!(custom, &hrs, Point::new(23, 29), STAT_ONE_CHAR, disp);
        write_text!(
            "hrs/day on average.",
            Point::new(26 + (hrs.len() * 8) as i32, 31),
            disp
        );

        let ideal = round_format!(TICKET_GOAL as f32 / days_between(&end, &start) as f32);
        write_text!(custom, &ideal, Point::new(23, 45), NUMBER_CHAR, disp);
        write_text!(
            "ideal daily tickets.",
            Point::new(26 + (ideal.len() * 8) as i32, 47),
            disp
        );

        let days_left = format!(2, "{}", days_between(&end, &now) - 1);
        write_text!(
            custom,
            &days_left,
            Point::new(23, 61),
            STAT_THREE_CHAR,
            disp
        );
        write_text!(
            "days left.",
            Point::new(26 + (days_left.len() * 8) as i32, 63),
            disp
        );

        let on_track = round_format!(
            (TICKET_GOAL - ticket_count + TICKET_OFFSET) as f32 / days_between(&end, &now) as f32
        );

        write_text!(custom, &on_track, Point::new(23, 77), STAT_ONE_CHAR, disp);
        write_text!(
            "hrs/day to get on track.",
            Point::new(26 + (on_track.len() * 8) as i32, 79),
            disp
        );
    }
}

pub mod session {
    use core::{str::FromStr, sync::atomic::AtomicBool};

    use embassy_executor::Spawner;
    use embassy_rp::rtc::DateTime;
    use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
    use embassy_time::Timer;
    use embedded_graphics::{
        geometry::Point,
        image::Image,
        prelude::{Primitive, Size},
        Drawable,
    };
    use heapless::String;
    use log::error;
    use tinytga::Tga;

    use crate::{
        draw_rect, draw_rounded_rect, draw_tga, format,
        gui::{BLACK_FILL, CENTERED_TEXT},
        util::{Events, PROGRESS_BAR},
        wifi::RequestData,
        write_large_text, write_text, Display, EVENTS, TICKETS, TICKET_OFFSET, UPDATE_INTERVAL,
    };

    use super::BACKGROUND;

    pub static ON_SCREEN: Signal<ThreadModeRawMutex, bool> = Signal::new();
    pub static FLASH: AtomicBool = AtomicBool::new(true);
    pub static TRIGGERED: AtomicBool = AtomicBool::new(true);

    pub async fn init(spawner: &Spawner) {
        UPDATE_INTERVAL.store(1, core::sync::atomic::Ordering::Relaxed);
        ON_SCREEN.signal(true);
        spawner.spawn(flash_task()).unwrap();
    }

    #[embassy_executor::task]
    pub async fn flash_task() {
        let mut flashing = false;
        while ON_SCREEN.signaled() {
            EVENTS.send(Events::FlashSessionScreen(flashing)).await;
            flashing = !flashing;
            Timer::after_secs(1).await;
        }
    }

    pub async fn flash(flash: bool, disp: &mut Display<'_>) {
        if FLASH.load(core::sync::atomic::Ordering::Relaxed) {
            if flash {
                draw_rect!(Point::new(73, 40), Size::new(8, 10), BACKGROUND, disp);
            } else {
                write_large_text!(":", Point::new(73, 40), disp);
            }
        }
    }

    pub async fn update(disp: &mut Display<'_>, data: RequestData, now: DateTime) {
        let (elapsed, goal, paused) = match data {
            RequestData::Session(elapsed, goal, paused) => (elapsed, goal, paused),
            _ => {
                error!("[GUI] [Session] Recieved incorrect data!");
                return;
            }
        };

        draw_tga!(PROGRESS_BAR, Point::new(20, 53), disp);

        let display = match elapsed {
            0 => String::<4>::from_str("1:00").unwrap(),
            60 => String::<4>::from_str("DONE").unwrap(),
            _ => format!(4, "0:{:02}", 60 - elapsed),
        };

        if elapsed == 60 {
            FLASH.store(false, core::sync::atomic::Ordering::Relaxed);
            if TRIGGERED.load(core::sync::atomic::Ordering::Relaxed) {
                TICKETS.store(
                    TICKETS.load(core::sync::atomic::Ordering::Relaxed) + 1,
                    core::sync::atomic::Ordering::Relaxed,
                );
                TRIGGERED.store(false, core::sync::atomic::Ordering::Relaxed);
            }
        } else {
            TRIGGERED.store(true, core::sync::atomic::Ordering::Relaxed);
        }

        draw_rect!(Point::new(64, 40), Size::new(32, 10), BACKGROUND, disp);
        write_large_text!(&display, Point::new(80, 40), CENTERED_TEXT, disp);

        write_text!("Ticket No. ", Point::new(2, 118), disp);
        let tickets = format!(
            3,
            "{}",
            1 + TICKETS.load(core::sync::atomic::Ordering::Relaxed) - TICKET_OFFSET
        );
        write_large_text!(&tickets, Point::new(46, 114), disp);

        draw_rect!(Point::new(134, 118), Size::new(32, 8), BACKGROUND, disp);
        if paused {
            write_text!("Paused", Point::new(134, 118), disp);
        } else if elapsed < 60 {
            write_text!("Ongoing", Point::new(130, 118), disp);
        } else {
            write_text!("Finished", Point::new(126, 118), disp);
        }

        draw_rounded_rect!(
            Point::new(20, 53),
            Size::new((120. * (elapsed as f32 / 60.)) as u32, 6),
            Size::new(2, 2),
            BLACK_FILL,
            disp
        );

        write_text!(goal, Point::new(80, 62), CENTERED_TEXT, disp);
    }
}
