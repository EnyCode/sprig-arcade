#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use core::cell::RefCell;

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{self, DMA_CH0, PIN_23, PIN_25, PIO0, USB};
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

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
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
    Timer::after_nanos(20000).await;

    let clk = p.PIN_18;
    let mosi = p.PIN_19;
    let miso = p.PIN_16;
    let display_cs = p.PIN_20;
    //let cs = p.PIN_20;
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
    //let cs = Output::new(cs, Level::Low);

    let _bl = Output::new(bl, Level::High);

    let mut disp = ST7735::new(display_spi, dcx, rst, true, false, 160, 128);

    disp.init(&mut Delay).unwrap();
    disp.set_orientation(&Orientation::Landscape).unwrap();
    disp.clear(Rgb565::new(31, 59, 26)).unwrap();

    let tga: Tga<Rgb565> = Tga::from_slice(include_bytes!("assets/arcade.tga")).unwrap();

    Image::new(&tga, Point::new(30, 98))
        .draw(&mut disp)
        .unwrap();

    Rectangle::new(Point::new(0, 0), Size::new(20, 20))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 2))
        .draw(&mut disp)
        .unwrap();

    //disp.set_offset(0, 25);

    //let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    //Text::new(
    //    "Hello embedded_graphics \n + embassy + RP2040!",
    //    Point::new(20, 200),
    //    style,
    //)
    //.draw(&mut disp)
    //.unwrap();

    loop {}
}
