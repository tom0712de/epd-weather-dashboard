
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
#[path = "../wifi.rs"]
mod wifi;
#[path = "../constants.rs"]
pub mod constants;

#[path = "../weather_codes.rs"]
pub mod weather_codes;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use esp_radio;
use chrono::{Datelike, NaiveDate, Weekday};
use core::fmt::Write;
// epd import

use esp_hal::delay::Delay;
use esp_hal::spi::master::{Spi};
use embedded_graphics::{
    prelude::*,
    pixelcolor::BinaryColor,
    geometry::Point,
    mono_font::{MonoTextStyle},
    image::Image

};
use tinybmp::Bmp;
use esp_hal::spi;
use embedded_hal_bus::spi::ExclusiveDevice;
use epdsi::prelude::*;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::time::Rate;
use heapless::String;
//
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    esp_println::println!("Panic");
    loop {
    } } pub struct Weact37;
    impl EpdPanel for Weact37{
        // maybe switch
        const WIDTH: u32 =416;
        const HEIGHT: u32 = 240;
        const COLOR_MODE: ColorMode = ColorMode::BlackWhite;
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write($val);
        x
    }};
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32 -o unstable-hal -o alloc -o wifi -o embassy
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    // --- Wifi Init
    //
    //
    //
    // posibly need to make static
    let (controller, interfaces)  = esp_radio::wifi::new(
        peripherals.WIFI,
        esp_radio::wifi::ControllerConfig::default()).unwrap_or_else(|error|{
            println!("Error :{}", error);
            loop{
            }
        }

    );
    let rng = esp_hal::rng::Rng::new();
    //let wifi_controller = esp_radio::wifi::WifiController<'static>, controller;
    let wifi_interface = *mk_static!(esp_radio::wifi::Interface, interfaces.station);
    
    let cfg = embassy_net::Config::dhcpv4(embassy_net::DhcpConfig::default());

    let net_seed = u64::from(rng.random()) | u64::from(rng.random()) << 32;
    
    // Stores info about stack upd tcp and metadate 
    // 3 because there can only be 3 tcp sockets at once
    let resources = mk_static!(
        embassy_net::StackResources<3>,
        embassy_net::StackResources::<3>::new()
    );

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        cfg,
        resources,
        net_seed,


        );

      // TODO: Spawn some tasks
    spawner.spawn(wifi::connection(stack,controller).expect(""));
    spawner.spawn(wifi::net_task(runner).expect(""));
    esp_println::println!("acces website");
    wifi::wait_for_connection(stack).await;
    #[allow(clippy::large_futures)]
    


    // Wifi init END -----

    // EPD INIT ---------------
    //
    
    let mut delay = Delay::new();
    //Initialize SPI_Bus Pins and create SPI_Bus_wrapper
    //
    let spi_bus = Spi::new(
        peripherals.SPI2,
        spi::master::Config::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(spi::Mode::_0),
    )
    .unwrap()
    //CLK
    .with_sck(peripherals.GPIO18)
    //DIN
    .with_mosi(peripherals.GPIO23);
    
    let cs = Output::new(peripherals.GPIO33, Level::High, OutputConfig::default());

    // specify that the spi_bus has only one spi device connected
    let spi = ExclusiveDevice::new(spi_bus,cs,delay).unwrap();

    let busy_pin = Input::new(
        peripherals.GPIO22,
        InputConfig::default().with_pull(Pull::None),
    );

    let rst_pin = Output::new(peripherals.GPIO16, Level::Low, OutputConfig::default());
    let dc_pin = Output::new(peripherals.GPIO17, Level::Low, OutputConfig::default());
    let epd_bus = SpiBusWrapper::new(spi, dc_pin, rst_pin, busy_pin);
    //-----------

     //----------- Init Controller  and epdDriver
    let controller = Uc8253Controller::new(240,416)
        .with_refresh_mode(Uc8253RefreshMode::Full);
    let mut epd = EpdBuilder::<_ ,Weact37>::new(controller).build(epd_bus); 
    //-----------
    //
    //
    //

    loop{
        #[allow(clippy::large_futures)]
        let response = wifi::access_website(stack,net_seed).await.unwrap_or_default();
        epd.init(&mut delay).unwrap();

        let mut buffer: [u8; 12480]= [0xFF;12480]; 
        let mut pagebuffer= PageBuffer::new(& mut buffer,240,416,0);
        pagebuffer.set_rotation(epdsi::graphics::buffer::DisplayRotation::Rotate90);
        // draw
        
       //Rectangle::new(Point::new(10,10), Size::new(206,150))
         //   .into_styled(style)
          //  .draw(& mut pagebuffer).unwrap();

        let style = MonoTextStyle::new(&profont::PROFONT_18_POINT, BinaryColor::On);
        let mut temperature_max: String<16> = String::new(); 

        let mut temperature_min: String<16> = String::new(); 
        let mut precipitation_sum: String<16> = String::new(); 
        
        // Graphics Prep for each day
        for i in 0..4{
            temperature_max.clear();
            temperature_min.clear();
            precipitation_sum.clear();
            let date = response.daily.time[i].parse::<NaiveDate>().unwrap_or_default();
            let day = match date.weekday(){
                Weekday::Mon => "Mon",
                Weekday::Tue => "Tue",
                Weekday::Wed => "Wed",
                Weekday::Thu=> "Thu",
                Weekday::Fri=> "Fri",
                Weekday::Sat=> "Sat",
                Weekday::Sun=> "Sun",

            };
            let offset = i*104 + 52;
            write!(& mut temperature_max,"{:?}",response.daily.temperature_2m_max[i]).unwrap_or_default();// kinda
            write!(& mut temperature_min,"{:?}  ",response.daily.temperature_2m_min[i]).unwrap_or_default();
            write!(& mut precipitation_sum,"{:?}  ",response.daily.precipitation_sum[i]).unwrap_or_default();

            let w_code = response.daily.weather_code[i];
            let bmp_data = weather_codes::convert_code_to_image(w_code);
            let bmp = Bmp::from_slice(bmp_data).unwrap();
            embedded_graphics::text::Text::new(
                day, 
                Point::new(offset.try_into().unwrap(),20),
                style
                )
                .draw(& mut pagebuffer);


            embedded_graphics::text::Text::new(
                &temperature_max, 
                Point::new(offset.try_into().unwrap(),70),
                style
                )
                .draw(& mut pagebuffer);

            Image::new(&bmp, Point::new(offset.try_into().unwrap(),90)).draw(&mut pagebuffer).unwrap();

            embedded_graphics::text::Text::new(
                &temperature_min, 
                Point::new(offset.try_into().unwrap(),50),
                style
                )
                .draw(& mut pagebuffer);

            embedded_graphics::text::Text::new(
                &precipitation_sum, 
                Point::new(offset.try_into().unwrap(),142),
                style
                )
                .draw(& mut pagebuffer);

        
            }

                


        //buffer boilerplate
        epd.write_frame(ColorChannel::BlackWhite, pagebuffer.as_slice()).unwrap();
        epd.refresh(&mut delay).unwrap();

        epd.sleep(&mut delay).unwrap();

        //update every two hours
        Timer::after(Duration::from_secs(60*60*2)).await;
        }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}

