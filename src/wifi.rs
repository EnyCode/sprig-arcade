use core::str::{from_utf8, FromStr};

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
use log::{error, info};
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
    Irqs,
};

#[derive(Deserialize, Debug)]
pub struct StatsResponse {
    ok: bool,
    data: Option<StatsData>,
    error: Option<&'static str>,
}

#[derive(Deserialize, Debug)]
pub struct StatsData {
    pub sessions: u32,
    pub total: u32,
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

    info!("joining wifi");

    loop {
        //match control.join_open(WIFI_NETWORK).await { // for open networks
        match control
            .join_wpa2(env!("WIFI_NETWORK"), env!("WIFI_PASSWD"))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
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

    info!("waiting for DHCP...");
    Timer::after_nanos(20000).await;
    while !stack.is_config_up() {
        info!("checking DHCP");
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
        "DHCP is now up! {:?}",
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

    info!("waiting for link up...");
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

    info!("Link is up!");
    Timer::after_nanos(20000).await;

    info!("waiting for stack to be up...");
    Timer::after_nanos(20000).await;
    stack.wait_config_up().await;
    info!("Stack is up!");
    Timer::after_nanos(20000).await;

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(20, 49), Size::new(120, 6)),
        Size::new(2, 2),
    )
    .into_styled(fill)
    .draw(display)
    .unwrap();

    display.clear(Rgb565::new(31, 60, 27)).unwrap();

    stack
}

pub async fn get_hours(stack: &'static Stack<cyw43::NetDriver<'static>>) -> Result<u32, Error> {
    static RX_BUF: StaticCell<[u8; 8192]> = StaticCell::new();
    let rx_buffer = RX_BUF.init([0; 8192]);
    //let mut tls_read_buffer = [0; 16640];
    //let mut tls_write_buffer = [0; 16640];

    let client_state = TcpClientState::<1, 1024, 1024>::new();
    let tcp_client = TcpClient::new(stack, &client_state);
    let dns_client = DnsSocket::new(stack);

    // TODO: use tls
    /*let tls_config = TlsConfig::new(
        seed,
        &mut tls_read_buffer,
        &mut tls_write_buffer,
        TlsVerify::None,
    );*/

    let mut http_client = HttpClient::new(&tcp_client, &dns_client);

    let mut url = String::<50>::new();
    url.push_str("http://hackhour.hackclub.com/api/stats/")
        .unwrap();
    url.push_str(env!("SLACK_ID")).unwrap();
    info!("{:?}", url);

    let mut req = http_client
        .request(reqwless::request::Method::GET, &url)
        .await?;

    let mut auth = String::<46>::from_str("Bearer ").unwrap();
    auth.push_str(env!("API_TOKEN")).unwrap();
    let header = [("Authorization", auth.as_str())];

    req = req.headers(&header);

    let resp = req.send(rx_buffer).await?;

    //info!("{:?}", rx_buffer);

    info!("made request");
    Timer::after_nanos(20000).await;

    let body: StatsResponse = serde_json_core::from_slice(resp.body().read_to_end().await.unwrap())
        .unwrap()
        .0;
    info!("response: {:?}", body);
    Timer::after_nanos(20000).await;

    info!("connecting to {}", &url);
    Timer::after_nanos(20000).await;

    if body.ok {
        Ok(body.data.unwrap().sessions)
    } else {
        error!(
            "Recieved the response, but it failed with error: {:?}",
            body.error.unwrap()
        );
        Err(Error::AlreadySent)
    }
}
