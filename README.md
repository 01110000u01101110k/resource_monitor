# resource monitor

It's a simple resource monitor. At the moment - it can get the temperature of the CPU and video card. 

The program has two implementations: multi-threaded and single-threaded. I recommend using the single-threaded one, as it is more economical and does not load the processor. The performance of the multithreaded and single-threaded implementations is now the same, due to the fact that it now uses nvml instead of nvidia-smi to obtain data on the temperature of the video card (nvidia-smi had a rather poor performance, the data request through it took from 400 to 1000 ms, which caused noticeable rendering delays in the single-threaded implementation).

As a ui used - egui/eframe, with the help of them 2 graphs drawing. 

At the moment, only windows implementation is supported, and also takes into account the peculiarities of my own laptop, so on some other configurations it may not work correctly, or not at all.