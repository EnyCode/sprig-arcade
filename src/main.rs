#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use core::cell::RefCell;

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_rp::gpio::{Input, Level, Output};
use embassy_rp::peripherals::{self, DMA_CH0, PIN_23, PIN_25, PIN_6, PIN_8, PIO0, USB};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::spi::{self, Spi};
use embassy_rp::spi::{Blocking, Phase, Polarity};
use embassy_rp::usb::{self, Driver};
use embassy_rp::{bind_interrupts, config};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::{Delay, Timer};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::primitives::{Primitive, PrimitiveStyle, Rectangle};
use embedded_graphics::Drawable;
use heapless::Vec;
use log::info;
use st7735_lcd::{Orientation, ST7735};
use tinytga::Tga;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<peripherals::USB>;
});

type Display = ST7735<
    SpiDeviceWithConfig<
        NoopRawMutex,
        Spi<peripherals::SPI0, Blocking>,
        Output<peripherals::PIN_20>,
    >,
    Output<peripherals::PIN_22>,
    Output<peripherals::PIN_26>,
>;

// we import everything here to avoid repeats and for ease of use
// makes it easier to eventually move to a fixed memory location if their all together (probably)
const ARCADE_LOGO: &'static [u8; 2347] = include_bytes!("assets/arcade.tga");
const BUTTONS: &'static [u8; 1942] = include_bytes!("assets/buttons.tga");
const BTN: &'static [u8; 204] = include_bytes!("assets/btn.tga");
const SELECTED_BTN: &'static [u8; 204] = include_bytes!("assets/selected_btn.tga");

const HOME_ICON: &'static [u8; 190] = include_bytes!("assets/buttons/home.tga");
const SESSION_ICON: &'static [u8; 232] = include_bytes!("assets/buttons/session.tga");
const LEADERBOARD_ICON: &'static [u8; 160] = include_bytes!("assets/buttons/leaderboard.tga");
const PROJECTS_ICON: &'static [u8; 182] = include_bytes!("assets/buttons/projects.tga");
const WISHLIST_ICON: &'static [u8; 218] = include_bytes!("assets/buttons/wishlist.tga");
const SHOP_ICON: &'static [u8; 204] = include_bytes!("assets/buttons/shop.tga");
const ERRORS_ICON: &'static [u8; 212] = include_bytes!("assets/buttons/errors.tga");

const HOME_SELECTED_ICON: &'static [u8; 190] = include_bytes!("assets/buttons/home_selected.tga");
const SESSION_SELECTED_ICON: &'static [u8; 232] =
    include_bytes!("assets/buttons/session_selected.tga");
const LEADERBOARD_SELECTED_ICON: &'static [u8; 160] =
    include_bytes!("assets/buttons/leaderboard_selected.tga");
const PROJECTS_SELECTED_ICON: &'static [u8; 182] =
    include_bytes!("assets/buttons/projects_selected.tga");
const WISHLIST_SELECTED_ICON: &'static [u8; 218] =
    include_bytes!("assets/buttons/wishlist_selected.tga");
const SHOP_SELECTED_ICON: &'static [u8; 204] = include_bytes!("assets/buttons/shop_selected.tga");
const ERRORS_SELECTED_ICON: &'static [u8; 212] =
    include_bytes!("assets/buttons/errors_selected.tga");

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
// TODO: priotize easy updating or efficient updating
// e.g. draw full button thing then draw selected stuff
// or draw default stuff on the old one and selected stuff on the new one
async fn update_ui(
    disp: &'static mut Display,
    left: Input<'static, PIN_6>,
    right: Input<'static, PIN_8>,
) {
    loop {
        Timer::after_secs(1).await;
    }
}

pub enum NavButton {
    None,
    Home,
    Session,
    Leaderboard,
    Projects,
    Wishlist, // TODO: merge wishlist with shop and make a settings page?
    Shop,
    // TODO: hide this by default
    Errors,
}

impl NavButton {
    pub fn pos(&self) -> Point {
        match self {
            NavButton::Home => Point::new(23, 0),
            NavButton::Session => Point::new(39, 0),
            NavButton::Leaderboard => Point::new(55, 0),
            NavButton::Projects => Point::new(71, 0),
            NavButton::Wishlist => Point::new(87, 0),
            NavButton::Shop => Point::new(103, 0),
            NavButton::Errors => Point::new(119, 0),
            NavButton::None => Point::new(0, 0),
        }
    }

    pub fn icon(&self) -> Tga<Rgb565> {
        Tga::from_slice(match self {
            NavButton::Home => HOME_ICON,
            NavButton::Session => SESSION_ICON,
            NavButton::Leaderboard => LEADERBOARD_ICON,
            NavButton::Projects => PROJECTS_ICON,
            NavButton::Wishlist => WISHLIST_ICON,
            NavButton::Shop => SHOP_ICON,
            NavButton::Errors => ERRORS_ICON,
            NavButton::None => HOME_ICON,
        })
        .unwrap()
    }

    pub fn selected_icon(&self) -> Tga<Rgb565> {
        Tga::from_slice(match self {
            NavButton::Home => HOME_SELECTED_ICON,
            NavButton::Session => SESSION_SELECTED_ICON,
            NavButton::Leaderboard => LEADERBOARD_SELECTED_ICON,
            NavButton::Projects => PROJECTS_SELECTED_ICON,
            NavButton::Wishlist => WISHLIST_SELECTED_ICON,
            NavButton::Shop => SHOP_SELECTED_ICON,
            NavButton::Errors => ERRORS_SELECTED_ICON,
            NavButton::None => HOME_SELECTED_ICON,
        })
        .unwrap()
    }

    pub fn icon_pos(&self) -> Point {
        match self {
            NavButton::Home => Point::new(28, 2),
            NavButton::Session => Point::new(43, 1),
            NavButton::Leaderboard => Point::new(55, 0),
            NavButton::Projects => Point::new(71, 0),
            NavButton::Wishlist => Point::new(87, 0),
            NavButton::Shop => Point::new(103, 0),
            NavButton::Errors => Point::new(119, 0),
            NavButton::None => Point::new(0, 0),
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    for _ in 0..2 {
        info!(".");
        Timer::after_secs(1).await;
    }
    info!("Launched Arcade Sprig!");
    // TODO: remove logging, dont know why its needed but it works
    Timer::after_nanos(20000).await;

    let clk = p.PIN_18;
    let mosi = p.PIN_19;
    let miso = p.PIN_16;
    let display_cs = p.PIN_20;
    let dcx = p.PIN_22;
    let rst = p.PIN_26;
    let bl = p.PIN_17;

    let mut display_config = spi::Config::default();
    display_config.frequency = 64_000_000;
    display_config.phase = Phase::CaptureOnSecondTransition;
    display_config.polarity = Polarity::IdleHigh;

    let spi: Spi<'_, _, Blocking> =
        Spi::new_blocking(p.SPI0, clk, mosi, miso, display_config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(
        &spi_bus,
        Output::new(display_cs, Level::High),
        display_config,
    );

    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);

    let _bl = Output::new(bl, Level::High);

    let mut disp = ST7735::new(display_spi, dcx, rst, true, false, 160, 128);

    disp.init(&mut Delay).unwrap();
    disp.set_orientation(&Orientation::Landscape).unwrap();
    disp.clear(Rgb565::new(31, 59, 26)).unwrap();

    // BOILERPLATE MARK

    let logo: Tga<Rgb565> = Tga::from_slice(ARCADE_LOGO).unwrap();
    let buttons: Tga<Rgb565> = Tga::from_slice(BUTTONS).unwrap();
    let btn: Tga<Rgb565> = Tga::from_slice(BTN).unwrap();
    let selected_btn: Tga<Rgb565> = Tga::from_slice(SELECTED_BTN).unwrap();

    Image::new(&logo, Point::new(30, 98))
        .draw(&mut disp)
        .unwrap();

    Image::new(&buttons, Point::new(23, 0))
        .draw(&mut disp)
        .unwrap();

    let active = NavButton::Home;
    let selected = NavButton::Session;

    Image::new(&selected_btn, selected.pos())
        .draw(&mut disp)
        .unwrap();

    Image::new(&selected.selected_icon(), selected.icon_pos())
        .draw(&mut disp)
        .unwrap();

    info!("Done!");
    // TODO: remove logging, dont know why its needed but it works
    Timer::after_nanos(20000).await;

    loop {}
}
