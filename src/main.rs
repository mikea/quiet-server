use clap::Parser;
use serde_json::Value;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 4, help = "Minimum fan speed percentage")]
    min_fan: i32,

    #[arg(long, default_value_t = 100, help = "Maximum fan speed percentage")]
    max_fan: i32,

    #[arg(
        long,
        default_value_t = 40.0,
        help = "Temperature at which fans run at minimum speed"
    )]
    min_temp: f64,

    #[arg(
        long,
        default_value_t = 90.0,
        help = "Temperature at which fans run at maximum speed"
    )]
    max_temp: f64,

    #[arg(
        long,
        default_value_t = 4.0,
        help = "Power curve exponent (decrease for cooler server, increase for quieter)"
    )]
    temp_pow: f64,
}

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

#[allow(clippy::cast_possible_truncation)]
fn determine_fan_level(temp: f64, args: &Args) -> i32 {
    let x = ((temp - args.min_temp) / (args.max_temp - args.min_temp)).clamp(0.0, 1.0);
    let fan_level =
        x.powf(args.temp_pow) * f64::from(args.max_fan - args.min_fan) + f64::from(args.min_fan);
    fan_level.round() as i32
}

fn set_fan(fan_level: i32) {
    // manual fan control
    Command::new("ipmitool")
        .args(["raw", "0x30", "0x30", "0x01", "0x00"])
        .output()
        .expect("Failed to set manual fan control");

    // set fan level
    let hex_fan = format!("{fan_level:#x}");
    Command::new("ipmitool")
        .args(["raw", "0x30", "0x30", "0x02", "0xff", &hex_fan])
        .output()
        .expect("Failed to set fan level");
}

fn main() {
    let args = Args::parse();
    let temp = get_temp();
    let fan = determine_fan_level(temp, &args);
    println!("temp {temp} fan {fan}");
    set_fan(fan);
}
