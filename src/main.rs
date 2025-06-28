use clap::Parser;
use std::process::Command;
use std::thread;
use std::time::Duration;

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
    
    #[arg(
        short,
        long,
        help = "Print temperature for each package and the resulting temperature"
    )]
    verbose: bool,

    #[arg(
        short,
        long,
        help = "Show what would be done without actually setting fan speeds"
    )]
    dry_run: bool,

    #[arg(
        short,
        long,
        default_value_t = 5.0,
        help = "Interval in seconds between fan speed adjustments"
    )]
    interval: f64,
}

fn get_temp(verbose: bool) -> f64 {
    let sensors = lm_sensors::Initializer::default().initialize().expect("Failed to initialize LM sensors");
    let mut max_temp: f64 = 0.0;
    let mut package_temps = Vec::new();
    
    // Iterate through all chips and find coretemp sensors
    for chip in sensors.chip_iter(None) {
        if let Some(Ok(prefix)) = chip.prefix() {
            if prefix.contains("coretemp") {
                // Look for Package temperature features
                for feature in chip.feature_iter() {
                    if let Ok(label) = feature.label() {
                        if label.contains("Package") {
                            // Get all subfeatures for this feature
                            for subfeature in feature.sub_feature_iter() {
                                if let Ok(lm_sensors::Value::TemperatureInput(temp_val)) = subfeature.value() {
                                    if verbose {
                                        package_temps.push((prefix.to_string(), label.clone(), temp_val));
                                    }
                                    max_temp = max_temp.max(temp_val);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    if verbose {
        for (chip, label, temp) in &package_temps {
            println!("{chip} - {label}: {temp:.1}°C");
        }
        println!("Effective temperature for calculation: {max_temp:.1}°C");
    }
    
    assert!(!(max_temp == 0.0), "No CPU temperature sensors found");
    
    max_temp
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
    let interval = Duration::from_secs_f64(args.interval);
    
    loop {
        let temp = get_temp(args.verbose);
        let fan = determine_fan_level(temp, &args);
        if args.verbose || args.dry_run {
            let prefix = if args.dry_run { "[DRY RUN] " } else { "" };
            println!("{prefix}Setting fan speed to {fan}% based on {temp:.1}°C");
        }
        if !args.dry_run {
            set_fan(fan);
        }
        
        thread::sleep(interval);
    }
}
