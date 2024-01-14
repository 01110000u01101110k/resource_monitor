use std::process::Command;

pub fn get_gpu_current_celsius_temperature() -> f32 {
    let data = Command::new("powershell")
        .args(&[
            "/C",
            r"nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader"
        ])
        .output()
        .expect("something went wrong");

    let data = String::from_utf8(data.stdout).unwrap();

    data.trim_end().parse::<f32>().unwrap()
}

/*pub fn get_gpu_current_celsius_temperature() -> f32 {
    let data = Command::new("powershell")
        .args(&[
            "/C",
            r"nvidia-smi -q -d TEMPERATURE"
        ])
        .output()
        .expect("something went wrong");

    let mut data = String::from_utf8(data.stdout).unwrap();

    let current_temp_index = data.find("GPU Current Temp").unwrap();

    let mut drained_data: String = data.drain(current_temp_index..).collect();

    let dots_index = drained_data.find(":").unwrap();

    let end_index = drained_data.find("C\r\n").unwrap();

    let final_str: String = drained_data.drain(dots_index + 1..end_index).collect();

    let gpu_temperature: f32 = final_str.trim().parse().unwrap();

    gpu_temperature
}*/