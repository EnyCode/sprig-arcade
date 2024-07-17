#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use core::any::Any;
use core::cell::RefCell;
use core::future::IntoFuture;

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_futures::select::{select, select3, select4};
use embassy_net::Stack;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{AnyPin, Input, Level, Output};
use embassy_rp::peripherals::{
    self, DMA_CH0, PIN_14, PIN_15, PIN_23, PIN_25, PIN_6, PIN_8, PIO0, USB,
};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::spi::{self, Spi};
use embassy_rp::spi::{Blocking, Phase, Polarity};
use embassy_rp::usb::{self, Driver};
use embassy_sync::blocking_mutex::raw::{
    CriticalSectionRawMutex, NoopRawMutex, ThreadModeRawMutex,
};
use embassy_sync::blocking_mutex::Mutex as BlockingMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::primitives::{Primitive, PrimitiveStyle, Rectangle};
use embedded_graphics::Drawable;
use gui::nav::{update_active, update_selected};
use heapless::Vec;
use log::info;
use st7735_lcd::{Orientation, ST7735};
use tinytga::Tga;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<peripherals::USB>;
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

mod gui;
mod wifi;

pub const TICKET_GOAL: u16 = 120;

// we import everything here to avoid repeats and for ease of use
// makes it easier to eventually move to a fixed memory location if their all together (probably)
const ARCADE_LOGO: &'static [u8; 2347] = include_bytes!("../assets/arcade.tga");
const BUTTONS: &'static [u8; 1942] = include_bytes!("../assets/buttons.tga");
const BTN: &'static [u8; 204] = include_bytes!("../assets/btn.tga");
const SELECTED_BTN: &'static [u8; 204] = include_bytes!("../assets/selected_btn.tga");
const ACTIVE_BTN: &'static [u8; 204] = include_bytes!("../assets/active_btn.tga");

const HOME_ICON: &'static [u8; 190] = include_bytes!("../assets/buttons/home.tga");
const SESSION_ICON: &'static [u8; 232] = include_bytes!("../assets/buttons/session.tga");
const LEADERBOARD_ICON: &'static [u8; 160] = include_bytes!("../assets/buttons/leaderboard.tga");
const PROJECTS_ICON: &'static [u8; 182] = include_bytes!("../assets/buttons/projects.tga");
const WISHLIST_ICON: &'static [u8; 218] = include_bytes!("../assets/buttons/wishlist.tga");
const SHOP_ICON: &'static [u8; 204] = include_bytes!("../assets/buttons/shop.tga");
const ERRORS_ICON: &'static [u8; 212] = include_bytes!("../assets/buttons/errors.tga");

type Display<'a> = ST7735<
    SpiDeviceWithConfig<
        'a,
        CriticalSectionRawMutex,
        Spi<'a, peripherals::SPI0, Blocking>,
        Output<'a, peripherals::PIN_20>,
    >,
    Output<'a, peripherals::PIN_22>,
    Output<'a, peripherals::PIN_26>,
>;

// TODO: double check length
static EVENTS: Channel<ThreadModeRawMutex, Events, 8> = Channel::new();

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
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
    Placeholder,
}

#[embassy_executor::task]
// TODO: priotize easy updating or efficient updating
// e.g. draw full button thing then draw selected stuff
// or draw default stuff on the old one and selected stuff on the new one
async fn input_task(
    mut up: Input<'static, AnyPin>,
    mut down: Input<'static, AnyPin>,
    mut left: Input<'static, AnyPin>,
    mut right: Input<'static, AnyPin>,
    mut a: Input<'static, AnyPin>,
    mut b: Input<'static, AnyPin>,
) {
    debug!("Starting input event task.");
    Timer::after_nanos(20000).await;
    loop {
        let previous = [
            a.is_high(),
            b.is_high(),
            up.is_high(),
            down.is_high(),
            left.is_high(),
            right.is_high(),
        ];

        let result = select3(
            a.wait_for_any_edge(),
            b.wait_for_any_edge(),
            select4(
                up.wait_for_any_edge(),
                down.wait_for_any_edge(),
                left.wait_for_any_edge(),
                right.wait_for_any_edge(),
            ),
        )
        .await;

        // debouncing
        Timer::after_millis(20).await;

        let change = match result {
            embassy_futures::select::Either3::First(_) => Button::A,
            embassy_futures::select::Either3::Second(_) => Button::B,
            embassy_futures::select::Either3::Third(result) => match result {
                embassy_futures::select::Either4::First(_) => Button::Up,
                embassy_futures::select::Either4::Second(_) => Button::Down,
                embassy_futures::select::Either4::Third(_) => Button::Left,
                embassy_futures::select::Either4::Fourth(_) => Button::Right,
            },
        };

        let button = match change {
            Button::A => a.is_high(),
            Button::B => b.is_high(),
            Button::Up => up.is_high(),
            Button::Down => down.is_high(),
            Button::Left => left.is_high(),
            Button::Right => right.is_high(),
        };

        let event = match button {
            true => Events::ButtonReleased(change),
            false => Events::ButtonPressed(change),
        };

        // changed button

        EVENTS.send(event).await;
        //info!("Button pressed! {:?}", change);
        //Timer::after_nanos(20000).await;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn icon_pos(&self) -> Point {
        match self {
            NavButton::Home => Point::new(28, 2),
            NavButton::Session => Point::new(43, 1),
            NavButton::Leaderboard => Point::new(60, 3),
            NavButton::Projects => Point::new(76, 1),
            NavButton::Wishlist => Point::new(91, 2),
            NavButton::Shop => Point::new(107, 0),
            NavButton::Errors => Point::new(124, 1),
            NavButton::None => Point::new(0, 0),
        }
    }

    pub fn right(&self) -> Self {
        match self {
            NavButton::Home => NavButton::Session,
            NavButton::Session => NavButton::Leaderboard,
            NavButton::Leaderboard => NavButton::Projects,
            NavButton::Projects => NavButton::Wishlist,
            NavButton::Wishlist => NavButton::Shop,
            NavButton::Shop => NavButton::Errors,
            NavButton::Errors => NavButton::Home,
            NavButton::None => NavButton::Home,
        }
    }

    pub fn left(&self) -> Self {
        match self {
            NavButton::Home => NavButton::Errors,
            NavButton::Session => NavButton::Home,
            NavButton::Leaderboard => NavButton::Session,
            NavButton::Projects => NavButton::Leaderboard,
            NavButton::Wishlist => NavButton::Projects,
            NavButton::Shop => NavButton::Wishlist,
            NavButton::Errors => NavButton::Shop,
            NavButton::None => NavButton::Home,
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
    // TODO: remove logging wait thing, dont know why its needed but it works
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
    let spi_bus: BlockingMutex<CriticalSectionRawMutex, _> = BlockingMutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(
        &spi_bus,
        Output::new(display_cs, Level::High),
        display_config,
    );

    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);

    let _bl = Output::new(bl, Level::High);

    let mut disp: Display = ST7735::new(display_spi, dcx, rst, true, false, 160, 128);

    disp.init(&mut Delay).unwrap();
    disp.set_orientation(&Orientation::Landscape).unwrap();
    disp.clear(Rgb565::new(31, 60, 27)).unwrap();

    // BOILERPLATE MARK

    wifi::setup(
        &spawner, p.PIN_23, p.PIN_25, p.PIO0, p.PIN_24, p.PIN_29, p.DMA_CH0,
    )
    .await;

    info!("Done w/ wifi");
    Timer::after_secs(1).await;

    spawner
        .spawn(input_task(
            Input::new(AnyPin::from(p.PIN_5), embassy_rp::gpio::Pull::Up),
            Input::new(AnyPin::from(p.PIN_7), embassy_rp::gpio::Pull::Up),
            Input::new(AnyPin::from(p.PIN_6), embassy_rp::gpio::Pull::Up),
            Input::new(AnyPin::from(p.PIN_8), embassy_rp::gpio::Pull::Up),
            Input::new(AnyPin::from(p.PIN_14), embassy_rp::gpio::Pull::Up),
            Input::new(AnyPin::from(p.PIN_15), embassy_rp::gpio::Pull::Up),
        ))
        .unwrap();

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

    let mut active = NavButton::Home;
    let mut selected = NavButton::Session;

    Image::new(&selected_btn, selected.pos())
        .draw(&mut disp)
        .unwrap();

    Image::new(&selected.icon(), selected.icon_pos())
        .draw(&mut disp)
        .unwrap();

    info!("Done!");
    Timer::after_nanos(20000).await;
    let tick_counter = 0;

    loop {
        Timer::after_millis(1).await;
        tick_counter += 1;

        match EVENTS.try_receive().unwrap_or(Events::Placeholder) {
            Events::ButtonPressed(button) => match button {
                Button::Left | Button::Right => {
                    selected = move_nav(&selected, &active, &button, &mut disp).await;
                }
                Button::A => active = select_btn(&selected, &active, &mut disp).await,
                _ => {}
            },
            Events::ButtonReleased(button) => {
                info!("released {:?}", button);
                info!("------------------");
            }
            _ => {}
        }

        if tick_counter > (1000 * 60 * 5) {
            info!("updating bar");
        }
    }
}

async fn move_nav(
    selected: &NavButton,
    active: &NavButton,
    button: &Button,
    disp: &mut Display<'_>,
) -> NavButton {
    info!("moving nav!");
    let prev_selected = selected.clone();
    let next = match button {
        Button::Left => selected.left(),
        Button::Right => selected.right(),
        _ => return selected.clone(),
    };
    update_selected(&next, &prev_selected, active, disp);

    next
}

async fn select_btn(selected: &NavButton, active: &NavButton, disp: &mut Display<'_>) -> NavButton {
    update_active(selected, active, disp);

    selected.clone()
}
