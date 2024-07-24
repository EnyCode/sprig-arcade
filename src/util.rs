use chrono::{DateTime, FixedOffset};
use heapless::String;

use crate::wifi::RequestData;

pub const ARCADE_LOGO: &'static [u8; 2347] = include_bytes!("../assets/arcade.tga");
pub const BUTTONS: &'static [u8; 1942] = include_bytes!("../assets/buttons.tga");
pub const BTN: &'static [u8; 204] = include_bytes!("../assets/btn.tga");
pub const SELECTED_BTN: &'static [u8; 204] = include_bytes!("../assets/selected_btn.tga");
pub const ACTIVE_BTN: &'static [u8; 204] = include_bytes!("../assets/active_btn.tga");

pub const HOME_ICON: &'static [u8; 190] = include_bytes!("../assets/buttons/home.tga");
pub const SESSION_ICON: &'static [u8; 232] = include_bytes!("../assets/buttons/session.tga");
pub const LEADERBOARD_ICON: &'static [u8; 160] =
    include_bytes!("../assets/buttons/leaderboard.tga");
pub const PROJECTS_ICON: &'static [u8; 182] = include_bytes!("../assets/buttons/projects.tga");
pub const WISHLIST_ICON: &'static [u8; 218] = include_bytes!("../assets/buttons/wishlist.tga");
pub const SHOP_ICON: &'static [u8; 204] = include_bytes!("../assets/buttons/shop.tga");
pub const ERRORS_ICON: &'static [u8; 212] = include_bytes!("../assets/buttons/errors.tga");

pub const TICKET_LARGE: &'static [u8; 471] = include_bytes!("../assets/ticket_large.tga");
pub const TICKET_SMALL: &'static [u8; 187] = include_bytes!("../assets/ticket_small.tga");

pub const PROGRESS_BAR: &'static [u8; 1033] = include_bytes!("../assets/session/progress.tga");

// TODO: if there is more side bars then move to similar system as top nav
pub const PROGRESS_SELECTED: &'static [u8; 710] =
    include_bytes!("../assets/home/progress_selected.tga");
pub const STATS_SELECTED: &'static [u8; 710] = include_bytes!("../assets/home/stats_selected.tga");

// TODO: replace legacy code with this
#[macro_export]
macro_rules! format {
    ($size:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        let mut string = heapless::String::<$size>::new();
        match core::write!(&mut string, $($arg)*) {
            Ok(_) => string,
            Err(err) => {
                log::error!("Failed to format string with error {:?}", err);
                String::<$size>::new()
            },
        }
    }};
}

// TODO: dont think this is needed
#[macro_export]
macro_rules! check {
    ($e:expr, $default:expr) => {{
        match $e {
            x if x.overflowing() => $default,
            x => x.0,
        }
    }};
}

#[macro_export]
macro_rules! draw_tga {
    ($constant:expr, $point:expr, $disp:expr) => {
        Image::new(&Tga::from_slice($constant).unwrap(), $point)
            .draw($disp)
            .unwrap();
    };
    (tga, $tga:expr, $point:expr, $disp:expr) => {
        Image::new(&$tga, $point).draw($disp).unwrap();
    };
}

#[macro_export]
macro_rules! write_text {
    ($text:expr, $point:expr, $disp:expr) => {
        embedded_graphics::text::Text::new($text, $point, crate::gui::BLACK_CHAR)
            .draw($disp)
            .unwrap();
    };
    ($text:expr, $point:expr, $style:expr, $disp:expr) => {
        embedded_graphics::text::Text::with_text_style(
            $text,
            $point,
            crate::gui::BLACK_CHAR,
            $style,
        )
        .draw($disp)
        .unwrap();
    };
    (custom, $text:expr, $point:expr, $char:expr, $disp:expr) => {
        embedded_graphics::text::Text::new($text, $point, $char)
            .draw($disp)
            .unwrap();
    };
    (custom, $text:expr, $point:expr, $char:expr, $style:expr, $disp:expr) => {
        embedded_graphics::text::Text::with_text_style($text, $point, $char, $style)
            .draw($disp)
            .unwrap();
    };
}

#[macro_export]
macro_rules! write_large_text {
    ($text:expr, $point:expr, $disp:expr) => {
        embedded_graphics::text::Text::new($text, $point, crate::gui::BLACK_NUMBER_CHAR)
            .draw($disp)
            .unwrap();
    };
    ($text:expr, $point:expr, $style:expr, $disp:expr) => {
        embedded_graphics::text::Text::with_text_style(
            $text,
            $point,
            crate::gui::BLACK_NUMBER_CHAR,
            $style,
        )
        .draw($disp)
        .unwrap();
    };
}

#[macro_export]
macro_rules! draw_rect {
    ($point:expr, $size:expr, $style:expr, $disp:expr) => {
        embedded_graphics::primitives::Rectangle::new($point, $size)
            .into_styled($style)
            .draw($disp)
            .unwrap();
    };
}

#[macro_export]
macro_rules! draw_rounded_rect {
    ($point:expr, $size:expr, $radii:expr, $style:expr, $disp:expr) => {
        embedded_graphics::primitives::RoundedRectangle::with_equal_corners(
            embedded_graphics::primitives::Rectangle::new($point, $size),
            $radii,
        )
        .into_styled($style)
        .draw($disp)
        .unwrap();
    };
}

// TODO: replace .unwrap() with this
#[macro_export]
macro_rules! unwrap {
    ($e:expr, $log:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => {
                log::error!("{:?} {:?}", log, err);
                return;
            }
        }
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
}

pub enum Events {
    ButtonPressed(Button),
    ButtonReleased(Button),
    DataUpdate(RequestData),
    RtcUpdate(DateTime<FixedOffset>),
    FlashSessionScreen(bool),
}
