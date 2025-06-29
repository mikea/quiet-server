use clap::Parser;
use ipmiraw::si::Ipmi;
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

    #[arg(
        short,
        long,
        help = "Force fan speed updates even when speed hasn't changed"
    )]
    force: bool,
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

fn set_fan(fan_level: i32) -> Result<(), Box<dyn std::error::Error>> {
    let ipmi = Ipmi::open("/dev/ipmi0")?;
    
    // manual fan control
    ipmi.cmd(0x30, 0x30, &[0x01, 0x00])?;
    
    // set fan level  
    ipmi.cmd(0x30, 0x30, &[0x02, 0xff, u8::try_from(fan_level)?])?;
    
    Ok(())
}

fn validate_ipmi() -> Result<(), Box<dyn std::error::Error>> {
    let ipmi = Ipmi::open("/dev/ipmi0")?;
    
    // Get device ID to verify IPMI is working
    ipmi.cmd(0x06, 0x01, &[])?;
    println!("IPMI device accessible");
    
    // Test getting fan speeds (this may fail on some systems, but we'll try)
    if let Err(e) = ipmi.cmd(0x30, 0x45, &[0x00]) {
        println!("Warning: Could not read fan status: {e}");
        println!("Fan control may not work on this system");
    } else {
        println!("Fan control commands appear to be supported");
    }
    
    Ok(())
}

fn main() {
    let args = Args::parse();
    
    // Validate IPMI functionality before starting
    if !args.dry_run {
        if let Err(e) = validate_ipmi() {
            eprintln!("IPMI validation failed: {e}");
            eprintln!("Make sure /dev/ipmi0 exists and you have proper permissions");
            std::process::exit(1);
        }
    }
    
    let interval = Duration::from_secs_f64(args.interval);
    let mut last_fan_speed = 0;
    
    loop {
        let temp = get_temp(args.verbose);
        let fan = determine_fan_level(temp, &args);
        
        let should_set_fan = args.force || last_fan_speed != fan;
        
        if args.verbose || args.dry_run {
            let prefix = if args.dry_run { "[DRY RUN] " } else { "" };
            if should_set_fan {
                println!("{prefix}Setting fan speed to {fan}% based on {temp:.1}°C");
            } else if args.verbose {
                println!("Fan speed unchanged at {fan}% (temp: {temp:.1}°C)");
            }
        }
        
        if !args.dry_run && should_set_fan {
            if let Err(e) = set_fan(fan) {
                eprintln!("Error setting fan speed: {e}");
                std::process::exit(1);
            }
        }
        
        last_fan_speed = fan;
        thread::sleep(interval);
    }
}
