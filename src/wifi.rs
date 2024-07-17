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
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex, ThreadModeRawMutex},
    mutex::Mutex,
};
use embassy_time::Timer;
use heapless::String;
use log::{error, info};
use rand::RngCore;
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    request::RequestBuilder,
};
use static_cell::StaticCell;

use crate::Irqs;

static STACK: Mutex<ThreadModeRawMutex, Stack<cyw43::NetDriver<'static>>> = Mutex::new();

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
) {
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

    let config = Config::dhcpv4(Default::default());

    let seed = rng.next_u64();

    static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<5>::new()),
        seed,
    ));

    spawner.spawn(net_task(stack)).unwrap();

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

    info!("waiting for DHCP...");
    Timer::after_nanos(20000).await;
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!(
        "DHCP is now up! {:?}",
        stack.config_v4().unwrap().address.address()
    );
    Timer::after_nanos(20000).await;

    info!("waiting for link up...");
    Timer::after_nanos(20000).await;
    while !stack.is_link_up() {
        Timer::after_millis(500).await;
    }
    info!("Link is up!");
    Timer::after_nanos(20000).await;

    info!("waiting for stack to be up...");
    Timer::after_nanos(20000).await;
    stack.wait_config_up().await;
    info!("Stack is up!");
    Timer::after_nanos(20000).await;
}

async fn get_stats() {
    let mut rx_buffer = [0; 8192];
    let mut tls_read_buffer = [0; 16640];
    let mut tls_write_buffer = [0; 16640];

    let stack = *STACK;

    let client_state = TcpClientState::<1, 1024, 1024>::new();
    let tcp_client = TcpClient::new(stack, &client_state);
    let dns_client = DnsSocket::new(stack);

    // TODO: use tls
    let tls_config = TlsConfig::new(
        seed,
        &mut tls_read_buffer,
        &mut tls_write_buffer,
        TlsVerify::None,
    );

    let mut http_client = HttpClient::new(&tcp_client, &dns_client);
    // combine "http://hackhour.hackclub.com/api/stats/" with env!("SLACK_ID")
    let mut url = String::<50>::new();
    url.push_str("http://hackhour.hackclub.com/api/stats/")
        .unwrap();
    url.push_str(env!("SLACK_ID")).unwrap();
    info!("{:?}", url);

    let mut req = match http_client
        .request(reqwless::request::Method::GET, &url)
        .await
    {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to make HTTP request: {:?}", e);
            Timer::after_nanos(20000).await;
            return; // handle the error
        }
    };

    let mut auth = String::<46>::from_str("Bearer ").unwrap();
    auth.push_str(env!("API_TOKEN")).unwrap();
    let header = [("Authorization", auth.as_str())];

    req = req.headers(&header);

    let resp = match req.send(&mut rx_buffer).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to send HTTP request: {:?}", e);
            Timer::after_nanos(20000).await;
            return; // handle the error;
        }
    };

    info!("made request");
    Timer::after_nanos(20000).await;

    let body = from_utf8(resp.body().read_to_end().await.unwrap()).unwrap();
    info!("response: {}", body);
    Timer::after_nanos(20000).await;

    info!("connecting to {}", &url);
    Timer::after_nanos(20000).await;
}
