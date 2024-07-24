use core::{
    ptr::addr_of_mut,
    str::{from_utf8, FromStr},
    sync::atomic::AtomicBool,
};

use chrono::{DateTime, FixedOffset, TimeZone};
use cyw43::State;
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Config, Stack, StackResources,
};
use embassy_rp::{
    clocks::RoscRng,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0},
    pio::Pio,
    Peripherals,
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex, ThreadModeRawMutex},
    mutex::{Mutex, MutexGuard},
    signal::Signal,
};
use embassy_time::Timer;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Primitive,
    text::Text,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    Drawable,
};
use heapless::String;
use log::{debug, error, info};
use rand::RngCore;
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    request::RequestBuilder,
    Error,
};
use serde::Deserialize;
use static_cell::StaticCell;

use crate::{
    gui::{BLACK_CHAR, CENTERED_TEXT},
    Irqs, EVENTS, TICKETS, UPDATE_INTERVAL,
};

pub static RUN: Signal<ThreadModeRawMutex, bool> = Signal::new();

#[derive(Deserialize, Debug)]
pub struct SessionResponse {
    ok: bool,
    data: Option<SessionData>,
    error: Option<&'static str>,
}

#[derive(Deserialize, Debug)]
pub struct SessionData {
    elapsed: u8,
    goal: &'static str,
    paused: bool,
}

#[derive(Deserialize, Debug)]
pub struct StatsResponse {
    ok: bool,
    data: Option<StatsData>,
    error: Option<&'static str>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StatsData {
    pub sessions: u32,
    pub total: u32,
}

#[derive(Deserialize, Debug)]
pub struct TimeData {
    pub datetime: &'static str,
}

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[derive(Clone, Copy, Debug)]
pub enum RequestType {
    Stats,
    Session,
}

#[derive(Debug)]
pub enum RequestData {
    // Ticket count
    Stats(u16),
    // Elapsed time, goal, paused
    Session(u8, &'static str, bool),
    None,
}

pub static REQUEST_TYPE: Mutex<ThreadModeRawMutex, RequestType> = Mutex::new(RequestType::Stats);

pub async fn setup(
    spawner: &Spawner,
    pwr_pin: PIN_23,
    cs_pin: PIN_25,
    pio_ch: PIO0,
    dio: PIN_24,
    clk: PIN_29,
    dma_ch: DMA_CH0,
    display: &mut crate::Display<'_>,
) -> &'static Stack<cyw43::NetDriver<'static>> {
    Text::with_text_style("Loading...", Point::new(80, 40), BLACK_CHAR, CENTERED_TEXT)
        .draw(display)
        .unwrap();

    let background = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::new(30, 57, 24))
        .build();

    let fill = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::new(1, 44, 23))
        .build();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(120, 6)),
        Size::new(2, 2),
    )
    .into_styled(background)
    .draw(display)
    .unwrap();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(5, 6)),
        Size::new(2, 2),
    )
    .into_styled(fill)
    .draw(display)
    .unwrap();

    let mut rng = RoscRng;

    let fw = include_bytes!("../firmware/43439A0.bin");
    let clm = include_bytes!("../firmware/43439A0_clm.bin");

    let pwr = Output::new(pwr_pin, Level::Low);
    let cs = Output::new(cs_pin, Level::High);
    let mut pio = Pio::new(pio_ch, Irqs);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, dio, clk, dma_ch);

    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());

    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(wifi_task(runner)).unwrap();

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(20, 6)),
        Size::new(2, 2),
    )
    .into_styled(fill)
    .draw(display)
    .unwrap();

    let config = Config::dhcpv4(Default::default());

    let seed = rng.next_u64();

    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<5>::new()),
        seed,
    ));

    spawner.spawn(net_task(stack)).unwrap();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(30, 6)),
        Size::new(2, 2),
    )
    .into_styled(fill)
    .draw(display)
    .unwrap();

    info!("[Wifi] Joining network");

    loop {
        //match control.join_open(WIFI_NETWORK).await { // for open networks
        match control
            .join_wpa2(env!("WIFI_NETWORK"), env!("WIFI_PASSWD"))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!(
                    "[Wifi] Failed to join with status {}, trying again",
                    err.status
                );
                Timer::after_nanos(20000).await;
            }
        }
    }

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(50, 6)),
        Size::new(2, 2),
    )
    .into_styled(fill)
    .draw(display)
    .unwrap();

    let mut i = 0;

    info!("[Wifi] Waiting for DHCP");
    Timer::after_nanos(20000).await;
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(20, 49), Size::new(50 + (50 * (i / 10)), 6)),
            Size::new(2, 2),
        )
        .into_styled(fill)
        .draw(display)
        .unwrap();
        if i < 10 {
            i += 1;
        }
    }

    info!(
        "[Wifi] DHCP is up with IP {}",
        stack.config_v4().unwrap().address.address()
    );
    Timer::after_nanos(20000).await;

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(100, 6)),
        Size::new(2, 2),
    )
    .into_styled(fill)
    .draw(display)
    .unwrap();

    info!("[Wifi] Waiting for link");
    Timer::after_nanos(20000).await;
    while !stack.is_link_up() {
        Timer::after_millis(500).await;
    }

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(110, 6)),
        Size::new(2, 2),
    )
    .into_styled(fill)
    .draw(display)
    .unwrap();

    info!("[Wifi] Link is up");
    Timer::after_nanos(20000).await;

    info!("[Wifi] Waiting for stack");
    Timer::after_nanos(20000).await;
    stack.wait_config_up().await;
    info!("[Wifi] Stack is up");
    Timer::after_nanos(20000).await;

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(120, 6)),
        Size::new(2, 2),
    )
    .into_styled(fill)
    .draw(display)
    .unwrap();

    display.clear(Rgb565::new(31, 60, 27)).unwrap();

    info!("[Wifi] Up and running");

    stack
}

#[embassy_executor::task]
pub async fn wifi_trigger() {
    loop {
        let mut i = 0;
        loop {
            Timer::after_secs(60).await;
            i += 1;
            if i == UPDATE_INTERVAL.load(core::sync::atomic::Ordering::Relaxed) {
                break;
            }
        }
        RUN.signal(true);
    }
}

#[embassy_executor::task]
pub async fn fetch_data(stack: &'static Stack<cyw43::NetDriver<'static>>) {
    let client_state = TcpClientState::<1, 1024, 1024>::new();
    let tcp_client = TcpClient::new(stack, &client_state);
    let dns_client = DnsSocket::new(stack);

    let mut http_client = HttpClient::new(&tcp_client, &dns_client);
    static mut RX_BUF: [u8; 8192] = [0; 8192];

    loop {
        unsafe {
            let rx_buffer = &mut RX_BUF;
            rx_buffer.fill(0);
            debug!("[Wifi] Fetching Hack Hour info");
            Timer::after_nanos(200000).await;

            debug!("got client");
            Timer::after_nanos(200000).await;

            let mut data = RequestData::None;
            {
                let mut url = String::<52>::new();
                let typ = *(REQUEST_TYPE.lock().await);
                match typ {
                    RequestType::Stats => {
                        url.push_str("http://hackhour.hackclub.com/api/stats/")
                            .unwrap();
                    }
                    RequestType::Session => {
                        url.push_str("http://hackhour.hackclub.com/api/session/")
                            .unwrap();
                    }
                }
                //url.push_str("http://hackhour.hackclub.com/api/stats/")
                //    .unwrap();
                url.push_str(env!("SLACK_ID")).unwrap();

                debug!("making request");
                Timer::after_nanos(200000).await;

                let mut req = http_client
                    .request(reqwless::request::Method::GET, &url)
                    .await
                    .unwrap();

                let mut header = String::<46>::from_str("Bearer ").unwrap();
                header.push_str(env!("API_TOKEN")).unwrap();
                let headers = [("Authorization", header.as_str())];

                req = req.headers(&headers);

                debug!("got headers");
                Timer::after_nanos(200000).await;

                let resp = req
                    .send(rx_buffer)
                    .await
                    .unwrap()
                    .body()
                    .read_to_end()
                    .await
                    .unwrap();

                debug!("sent request {:?}", from_utf8(resp));
                Timer::after_nanos(200000).await;

                match typ {
                    RequestType::Stats => {
                        let body: StatsResponse = match serde_json_core::from_slice(resp) {
                            Ok(b) => b.0,
                            Err(e) => {
                                error!(
                                    "[Wifi] Failed to parse the Hack Hour response. Is it down?",
                                );
                                error!(
                                    "[Wifi] Recieved response `{:?}` and failed with error `{:?}",
                                    from_utf8(resp),
                                    e
                                );
                                return;
                            }
                        };

                        if !body.ok {
                            error!(
                                "[Wifi] Hack Hour response failed. Are you authenticated properly?"
                            );
                            error!("[Wifi] Error is `{}`", body.error.unwrap());
                            return;
                        }
                        data = RequestData::Stats(body.data.clone().unwrap().sessions as u16);
                    }
                    RequestType::Session => {
                        let body: SessionResponse = match serde_json_core::from_slice(resp) {
                            Ok(b) => b.0,
                            Err(e) => {
                                error!(
                                    "[Wifi] Failed to parse the Hack Hour response. Is it down?",
                                );
                                error!(
                                    "[Wifi] Recieved response `{:?}` and failed with error `{:?}",
                                    from_utf8(resp),
                                    e
                                );
                                return;
                            }
                        };

                        if !body.ok {
                            error!(
                                "[Wifi] Hack Hour response failed. Are you authenticated properly?"
                            );
                            return;
                        }
                        let d = body.data.unwrap();
                        data = RequestData::Session(d.elapsed, d.goal, d.paused);
                    }
                }
            }

            debug!("got tickets");
            Timer::after_nanos(200000).await;
            debug!("making request");
            Timer::after_nanos(200000).await;

            EVENTS.send(crate::Events::DataUpdate(data)).await;
        }
        debug!("[Wifi] Sent event successfully!");
        RUN.reset();
        debug!("[Wifi] {:?}", RUN.signaled());
        RUN.wait().await;
        debug!("[Wifi] Starting again!");
    }
}

//#[embassy_executor::task]
pub async fn configure_rtc(stack: &'static Stack<cyw43::NetDriver<'static>>) {
    static RAN: AtomicBool = AtomicBool::new(false);
    if RAN.load(core::sync::atomic::Ordering::Relaxed) {
        error!("[Wifi] Tried to configure RTC twice!");
        return;
    }
    RAN.store(true, core::sync::atomic::Ordering::Relaxed);

    unsafe {
        debug!("[Wifi] Configuring RTC");
        let client_state = TcpClientState::<1, 1024, 1024>::new();
        let tcp_client = TcpClient::new(stack, &client_state);
        let dns_client = DnsSocket::new(stack);

        let tls_read = &mut [0u8; 16384];
        let tls_write = &mut [0u8; 16384];

        let mut http_client = HttpClient::new_with_tls(
            &tcp_client,
            &dns_client,
            TlsConfig::new(312, tls_read, tls_write, TlsVerify::None),
        );
        static mut RX_BUF: StaticCell<[u8; 8192]> = StaticCell::new();
        let rx_buffer = RX_BUF.init([0; 8192]);

        let mut req = match http_client
            .request(
                reqwless::request::Method::GET,
                "https://time.eny.hackclub.app/et",
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                error!("[Wifi] Failed to make time request. ");
                error!("[Wifi] Error is `{:?}`", e);
                Timer::after_nanos(4000000).await;
                return;
            }
        };

        debug!("made request");
        Timer::after_nanos(200000).await;

        debug!("filled buffer");

        let resp = req
            .send(rx_buffer)
            .await
            .unwrap()
            .body()
            .read_to_end()
            .await
            .unwrap();

        debug!("sent request");
        Timer::after_nanos(200000).await;

        let body: TimeData = match serde_json_core::from_slice(resp) {
            Ok(b) => b.0,
            Err(e) => {
                error!("[Wifi] Failed to parse time response. ");
                error!(
                    "[Wifi] Recieved response `{:?}` and failed with error `{:?}`",
                    from_utf8(resp),
                    e
                );
                return;
            }
        };

        debug!("[Wifi] Configured RTC");
        Timer::after_nanos(200000).await;

        EVENTS
            .send(crate::Events::RtcUpdate(
                DateTime::parse_from_rfc3339(body.datetime).unwrap(),
            ))
            .await;
    }
}
