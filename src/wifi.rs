
#![allow(
    clippy::large_stack_frames,
    reason = "Larger stack frames are expected for this embedded application"
)]
use serde::Deserialize;
use embassy_time::{Duration, Timer};
use heapless::String;
pub use crate::constants;
#[derive(Debug)]
pub enum Err{
    Network,
    InvalidUtf8,

}

#[derive(Debug,Default,Deserialize)]
pub struct Response{
    pub daily: ResponseDaily,
}
#[derive(Debug,Default,Deserialize)]
pub struct ResponseDaily {
    pub time: [String<16>;7],
    pub weather_code: [i16;7],
    pub temperature_2m_max:[f32;7],
    pub temperature_2m_min:[f32;7],
    pub precipitation_sum:[f32;7],

}
#[embassy_executor::task]
pub async fn connection(
    stack: embassy_net::Stack<'static>, 
    mut controller: esp_radio::wifi::WifiController<'static>,
    ) {
    loop{
        //check if connected
        if controller.is_connected(){
            Timer::after(Duration::from_millis(500)).await;

        }else{
            let station_config = esp_radio::wifi::Config::Station(
                esp_radio::wifi::sta::StationConfig::default()
                .with_ssid(constants::SSID)
                .with_password(constants::PASSWD.into())
                );
            match controller.set_config(&station_config){
                Ok(()) => (),
                Err(e) => {
                    esp_println::println!("Error while trying to connect to wifi: {}, trying again soon",e);
                    break;

                }

            }
            match controller.connect_async().await {
                Ok(_) => {
                    esp_println::println!("Wifi connected!");
                }
                Err(e) => {
                  esp_println::println!("Failed to connect to wifi: {e:?}");
                }
            }
        


        }
        
    }



}

#[embassy_executor::task]
pub async fn net_task(
    mut runner: embassy_net::Runner<'static,
    esp_radio::wifi::Interface<'static>>
    ) 
{
    runner.run().await;
} 
pub async fn access_website(stack: embassy_net::Stack<'static> ,tls_seed: u64) -> Result<Response,Err>{
    let mut rx_buffer = [0; 8192];
    let mut tx_buffer = [0; 8192];
    let dns = embassy_net::dns::DnsSocket::new(stack);
    let tcp_state = embassy_net::tcp::client::TcpClientState::<1, 4096, 4096>::new();
    let tcp = embassy_net::tcp::client::TcpClient::new(stack, &tcp_state);

    let tls = reqwless::client::TlsConfig::new(
        tls_seed,
        &mut rx_buffer,
        &mut tx_buffer,
        reqwless::client::TlsVerify::None,
    );
    let mut client = reqwless::client::HttpClient::new_with_tls(&tcp,&dns,tls);
    let mut buffer = [0u8; 8192];
    //let url = ".com";
    //embedded-tls doesnt support the same tls encryption or version as open-meteo
    let url = "http://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&daily=weather_code%2Ctemperature_2m_max%2Cprecipitation_sum%2Ctemperature_2m_min&timezone=GMT";
    let mut http_req = client
        .request(
            reqwless::request::Method::GET,
            url,
        )
        .await.map_err(|e|{esp_println::println!("wifi request error {:?}",e);Err::Network})?;
    let response = http_req.send(&mut buffer).await.map_err(|_|Err::Network)?;

    let res = response.body().read_to_end().await.map_err(|_|Err::Network)?;
    let (response, _ )= serde_json_core::from_slice::<Response>(res).map_err(|e|{esp_println::println!("Error from serde{:?}",e);Err::InvalidUtf8})?;
    Ok(response)
}

pub async fn wait_for_connection(stack: embassy_net::Stack<'_>) {
    esp_println::println!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    esp_println::println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            esp_println::println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}



