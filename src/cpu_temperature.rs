use std::process::Command;
use windows::{
    core::*, Win32::System::Ole::*, Win32::System::Variant::*,
    Win32::System::Wmi::*,
};

pub fn get_cpu_current_celsius_temperature_using_wmi(server: &IWbemServices) -> Vec<f32> {
    unsafe {
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

                match VarFormat(
                    &value,
                    None,
                    VARFORMAT_FIRST_DAY_SYSTEMDEFAULT,
                    VARFORMAT_FIRST_WEEK_SYSTEMDEFAULT,
                    0
                ) {
                    Ok(value) => {
                        if !value.is_empty() {
                            match value.to_string().parse::<f32>() {
                                Ok(value) => {
                                    result_arr.push(value - 273.15);
                                },
                                Err(_) => {}
                            }
                        }
                    },
                    Err(_) => {}
                }

                VariantClear(&mut value).unwrap();
            } else {
                break result_arr;
            }
        }
    }
}

pub fn get_cpu_current_celsius_temperature() -> f32 {
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

pub fn get_cpu_name(server: &IWbemServices) -> String {
    unsafe {
        let query = server.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from("select Name from Win32_Processor"),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        ).unwrap();
        
        loop {
            let mut row = [None; 1];
            let mut returned = 0;
            query.Next(WBEM_INFINITE, &mut row, &mut returned).ok().unwrap();

            if let Some(row) = &row[0] {
                let mut value = Default::default();
                row.Get(w!("Name"), 0, &mut value, None, None).unwrap();

                match VarFormat(
                    &value,
                    None,
                    VARFORMAT_FIRST_DAY_SYSTEMDEFAULT,
                    VARFORMAT_FIRST_WEEK_SYSTEMDEFAULT,
                    0
                ) {
                    Ok(value) => {
                        break value.to_string();
                    },
                    Err(_) => {}
                }

                VariantClear(&mut value).unwrap();
            } else {
                break "інформація відсутня".to_string();
            }
        }
    }
}