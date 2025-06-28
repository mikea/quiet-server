use std::process::Command;
use serde_json::Value;

const MIN_FAN: i32 = 5;
const MAX_FAN: i32 = 100;
const MIN_TEMP: f64 = 50.0; // fans at min at this temp
const MAX_TEMP: f64 = 80.0; // fans at max at this temp
const TEMP_POW: f64 = 3.0; // decrease for cooler server, increase for quieter

fn get_temp() -> f64 {
    let output = Command::new("/usr/bin/sensors")
        .arg("-j")
        .output()
        .expect("Failed to execute sensors command");
    
    let json_str = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let sensors: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");
    
    let temp0 = sensors["coretemp-isa-0000"]["Package id 0"]["temp1_input"]
        .as_f64()
        .expect("Failed to get temp0");
    let temp1 = sensors["coretemp-isa-0001"]["Package id 1"]["temp1_input"]
        .as_f64()
        .expect("Failed to get temp1");
    
    temp0.max(temp1)
}

fn determine_fan_level(temp: f64) -> i32 {
    let x = ((temp - MIN_TEMP) / (MAX_TEMP - MIN_TEMP)).clamp(0.0, 1.0);
    let fan_level = x.powf(TEMP_POW) * (MAX_FAN - MIN_FAN) as f64 + MIN_FAN as f64;
    fan_level.round() as i32
}

fn set_fan(fan_level: i32) {
    // manual fan control
    Command::new("ipmitool")
        .args(&["raw", "0x30", "0x30", "0x01", "0x00"])
        .output()
        .expect("Failed to set manual fan control");
    
    // set fan level
    let hex_fan = format!("{:#x}", fan_level);
    Command::new("ipmitool")
        .args(&["raw", "0x30", "0x30", "0x02", "0xff", &hex_fan])
        .output()
        .expect("Failed to set fan level");
}

fn main() {
    let temp = get_temp();
    let fan = determine_fan_level(temp);
    println!("temp {} fan {}", temp, fan);
    set_fan(fan);
}