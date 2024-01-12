use std::process::Command;
use std::time::{Duration, Instant};

use windows::{
    core::*, Win32::System::Com::*, Win32::System::Ole::*, Win32::System::Variant::*,
    Win32::System::Wmi::*,
};

fn get_cpu_current_celsius_temperature_using_wmi() -> Vec<f32> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).unwrap();

        CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        ).unwrap();

        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER).unwrap();

        let server = locator.ConnectServer(&BSTR::from("root\\cimv2"), None, None, None, 0, None, None).unwrap();

        let query = server.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from("select Temperature from Win32_PerfFormattedData_Counters_ThermalZoneInformation"),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        ).unwrap();

        let mut result_arr = Vec::new();
        
        loop {
            let mut row = [None; 1];
            let mut returned = 0;
            query.Next(WBEM_INFINITE, &mut row, &mut returned).ok().unwrap();

            if let Some(row) = &row[0] {
                let mut value = Default::default();
                row.Get(w!("Temperature"), 0, &mut value, None, None).unwrap();
                println!(
                    "{}",
                    VarFormat(
                        &value,
                        None,
                        VARFORMAT_FIRST_DAY_SYSTEMDEFAULT,
                        VARFORMAT_FIRST_WEEK_SYSTEMDEFAULT,
                        0
                    ).unwrap()
                );

                result_arr.push(
                    VarFormat(
                        &value,
                        None,
                        VARFORMAT_FIRST_DAY_SYSTEMDEFAULT,
                        VARFORMAT_FIRST_WEEK_SYSTEMDEFAULT,
                        0
                    ).unwrap().to_string().parse().unwrap()
                );

                VariantClear(&mut value).unwrap();
            } else {
                break result_arr;
            }
        }
    }
}

fn get_gpu_current_celsius_temperature() -> f32 {
    let data = Command::new("powershell")
        .args(&[
            "/C",
            r"nvidia-smi -q -d TEMPERATURE"
        ])
        .output()
        .expect("something went wrong");

    let mut data = String::from_utf8(data.stdout).unwrap();

    let current_temp_index = data.rfind("GPU Current Temp").unwrap();

    let mut drained_data: String = data.drain(current_temp_index..).collect();

    let dots_index = drained_data.find(":").unwrap();

    let end_index = drained_data.find("C\r\n").unwrap();

    let final_str: String = drained_data.drain(dots_index + 1..end_index).collect();

    let gpu_temperature: f32 = final_str.trim().parse().unwrap();

    gpu_temperature
}

fn get_cpu_current_celsius_temperature() -> f32 {
    let data = Command::new("powershell")
        .args(&[
            "/C",
            r"wmic /namespace:\\root\cimv2 PATH Win32_PerfFormattedData_Counters_ThermalZoneInformation get Temperature"
        ])
        .output()
        .expect("something went wrong");

    let mut data = String::from_utf8(data.stdout).unwrap();

    let temp_index = data.find("\r\n").unwrap();

    let drained_data: String = data.drain(temp_index..).collect();

    drained_data.trim().parse::<f32>().unwrap() - 273.15
}

fn main() {
    let update_time = Instant::now();
    //println!("cpu temperature: {}", get_cpu_current_celsius_temperature());
    get_cpu_current_celsius_temperature();

    println!("get_cpu_current_celsius_temperature {}", update_time.elapsed().as_millis());

    let update_time = Instant::now();
    //println!("cpu temperature ver: {:?}", get_cpu_current_celsius_temperature_using_wmi());
    get_cpu_current_celsius_temperature_using_wmi();

    println!("get_cpu_current_celsius_temperature_using_wmi {}", update_time.elapsed().as_millis());

    let update_time = Instant::now();
    //println!("gpu temperature: {}", get_gpu_current_celsius_temperature());

    get_gpu_current_celsius_temperature();


    println!("get_gpu_current_celsius_temperature {}", update_time.elapsed().as_millis());

    loop {
        
    }
}