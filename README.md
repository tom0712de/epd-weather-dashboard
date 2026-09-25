# epd weather dashboard 
- epd weather dashboard is a rust programm written for a esp32 devkit 1 and a WeActStudio EPD. 
- It display weather data for the next 4 days.
- The weather data is fetched from the openmeteo webAPI
# How to run 
- Plugin your Esp32 devkit 1
- Connect the ESP to your EPD(this project uses the WeActStudio 3,7 ich EPD display)
- SSID='replace-with-SSID' PASSWD='replace-with-PASSWD' cargo run --release

