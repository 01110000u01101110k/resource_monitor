# resource monitor

It's a simple resource monitor. At the moment - it can get the temperature of the CPU and video card. 

Multithreading support is implemented so that the ui does not freeze while receiving temperature data (on Windows receiving temperature using wmic and nvidia-smi, this is happening quite slowly approximately: 200 - 1000 ms for each). 
As a ui used - egui/eframe, with the help of them 2 graphs drawing. 

At the moment, only windows implementation is supported, and also takes into account the peculiarities of my own laptop, so on some other configurations it may not work correctly, or not at all.