## Wifi-Detector for ESP32-C6

A lightweight wifi-detector built in **RUST** aimed at finding networks on 2.4 GHz frequency and detecting how many active connections there are by analyzing packets sent to the different routers.

## Features
- Finding networks in the surrounding
- Determining how many connections there are to the network
- Blazingly fast 🏎️

## Set-up

- Button, 2-pin
- ESP32-C6
- Display with I2CDisplayInterface, (I used this: [here](https://www.electrokit.com/lcd-oled-0.91128x32px-i2c?gad_source=1&gad_campaignid=17338847491&gbraid=0AAAAAD_OrGNuEapbRIsUjDZs1R2Ye0dBs&gclid=Cj0KCQjwlYHBBhD9ARIsALRu09rzxQx6BiLrGd_APMhhHtWLVXMT_PDhILj-40uwpqJkI_Avsyt7gJ4aAh8wEALw_wcB))

1. The Button needs to be connected to GPIO20 and Ground (G)
2. The display needs to be connected to GPIO21, GPI022, 3.3V and Ground (G)
