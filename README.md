# Quiet Server

A Rust-based fan controller that automatically adjusts server fan speeds based on CPU package temperatures using IPMI and hardware sensors.

> **⚠️ EXPERIMENTAL SOFTWARE - USE AT YOUR OWN RISK**
> 
> This is experimental software that directly controls server hardware. Improper fan control can lead to overheating and hardware damage. While I run this in my own home lab, you should thoroughly test it with your specific hardware and monitor temperatures carefully. Always ensure you have proper cooling and temperature monitoring in place.

## Overview

This tool continuously monitors CPU package temperatures via lm-sensors and automatically adjusts fan speeds using IPMI commands. **The primary purpose is to make production servers quieter for home lab deployments** by providing intelligent fan curve control with configurable temperature thresholds and power curves to balance cooling performance with noise levels.

### Why This Tool Exists

Modern enterprise servers are designed for data center environments where noise is not a concern. In these facilities, servers run their fans at maximum speeds to ensure optimal cooling in dense rack deployments with controlled ambient temperatures. This results in noise levels of 60-80 dB or higher - comparable to a vacuum cleaner running continuously.

The rise of affordable refurbished enterprise servers has made powerful hardware accessible to home lab enthusiasts. Decommissioned servers from data centers can often be purchased for a fraction of their original cost, offering incredible value with features like:
- Multiple CPU sockets with high core counts
- Large memory capacity (often 128GB+ RAM)
- Enterprise-grade reliability with redundant power supplies and fans
- Hot-swappable drives, fans, and power supplies for zero-downtime maintenance
- Hardware RAID controllers with battery backup
- Multiple high-speed network interfaces (often 10GbE)
- Remote out-of-band management (IPMI/iDRAC/iLO) for complete control from anywhere
- ECC memory for data integrity
- Built-in hardware monitoring and alerting

However, when these servers are repurposed for home labs or small office environments, the excessive fan noise becomes a significant problem. This tool allows you to run enterprise-grade hardware in noise-sensitive environments by intelligently adjusting fan speeds based on actual thermal requirements rather than worst-case data center scenarios.

## Features

- **Automatic Temperature Monitoring**: Reads CPU package temperatures from coretemp sensors
- **Intelligent Fan Curves**: Configurable power curve with min/max temperature and fan speed settings
- **Native IPMI Integration**: Direct fan control via IPMI kernel interface (no external tools required)
- **Continuous Operation**: Runs as a daemon with configurable monitoring intervals
- **Verbose Logging**: Optional detailed output showing temperature readings and fan adjustments
- **Dry Run Mode**: Test configuration without actually changing fan speeds

## System Requirements

### Hardware
- Server with IPMI-compatible BMC (Baseboard Management Controller)
- CPU with coretemp temperature sensors
- Linux-based operating system

### Checking IPMI Support
Before installing, verify your server supports IPMI:

```bash
# Check if IPMI kernel modules are loaded
lsmod | grep ipmi

# Load IPMI modules if not present
sudo modprobe ipmi_devintf
sudo modprobe ipmi_si

# Test basic IPMI functionality
ipmitool chassis status

# Check if fan control is supported (should return raw data)
ipmitool raw 0x30 0x45 0x00

# Verify temperature sensors are accessible
ipmitool sdr type temperature
```

If these commands fail, your server may not support IPMI or require additional BMC configuration.

### Software Dependencies
- **lm-sensors**: Hardware monitoring library and utilities
- **libsensors-dev**: Development headers for lm-sensors
- **clang**: Required for building lm-sensors Rust bindings
- **IPMI kernel modules**: Required for native IPMI access (ipmi_devintf, ipmi_si)

### Rust Dependencies
- Rust 2024 edition or later
- See `Cargo.toml` for specific crate dependencies

## Installation

### 1. Install System Dependencies

#### Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install lm-sensors libsensors-dev libclang-dev
```

#### RHEL/CentOS/Fedora:
```bash
sudo dnf install lm_sensors lm_sensors-devel clang-devel
# or for older versions:
sudo yum install lm_sensors lm_sensors-devel clang-devel
```

### 2. Configure Hardware Sensors
```bash
# Detect and configure sensors
sudo sensors-detect --auto

# Load sensor modules
sudo modprobe coretemp

# Verify sensors are working
sensors
```

### 3. Configure IPMI Access
```bash
# Load IPMI kernel modules
sudo modprobe ipmi_devintf
sudo modprobe ipmi_si

# Verify IPMI device exists
ls -la /dev/ipmi*

# The tool requires root access or appropriate permissions on /dev/ipmi0
```

### 4. Build and Install
```bash
# Clone the repository
git clone <repository-url>
cd quiet-server

# Build the project
cargo build --release

# Install binary
sudo cp target/release/quiet-server /usr/local/bin/

# Or install via cargo
cargo install --path .
```

## Usage

### Basic Usage
```bash
# Run with default settings
quiet-server

# Run in dry-run mode to test configuration
quiet-server --dry-run

# Run with verbose output
quiet-server --verbose
```

### Configuration Options
```bash
quiet-server [OPTIONS]

Options:
  --min-fan <MIN_FAN>      Minimum fan speed percentage [default: 4]
  --max-fan <MAX_FAN>      Maximum fan speed percentage [default: 100]
  --min-temp <MIN_TEMP>    Temperature at which fans run at minimum speed [default: 40.0]
  --max-temp <MAX_TEMP>    Temperature at which fans run at maximum speed [default: 90.0]
  --temp-pow <TEMP_POW>    Power curve exponent [default: 4.0]
  -i, --interval <INTERVAL> Interval in seconds between adjustments [default: 5.0]
  -v, --verbose            Print detailed temperature and fan information
  -d, --dry-run           Show what would be done without changing fan speeds
  -f, --force             Force fan speed updates even when speed hasn't changed
  -s, --single            Run once and exit instead of continuous monitoring
  -h, --help              Print help information
```

### Example Configurations

#### Quiet Server (Lower Fan Speeds)
```bash
quiet-server --min-fan 10 --max-fan 60 --temp-pow 2.0
```

#### High Performance (Aggressive Cooling)
```bash
quiet-server --min-temp 35 --max-temp 75 --temp-pow 6.0
```

#### Testing Configuration
```bash
quiet-server --dry-run --verbose --interval 2.0
```

## Running as a Service

### systemd Service (Recommended)
An example systemd service file is provided in [`examples/quiet-server.service`](examples/quiet-server.service). Copy it to the system directory:

```bash
# Copy the service file
sudo cp examples/quiet-server.service /etc/systemd/system/

# Edit the service file to adjust paths and settings as needed
sudo systemctl edit quiet-server.service

# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable quiet-server
sudo systemctl start quiet-server

# Check status and logs
sudo systemctl status quiet-server
sudo journalctl -u quiet-server -f
```

The example service includes:
- Automatic restart on failure
- Verbose logging to systemd journal
- Security hardening settings
- Configurable fan speed limits (10-80% in the example)

## Troubleshooting

### No Temperature Sensors Found
- Verify coretemp module is loaded: `lsmod | grep coretemp`
- Check sensor detection: `sudo sensors-detect`
- Test sensors manually: `sensors`

### IPMI Commands Failing
- Verify IPMI access: `ipmitool chassis status`
- Check user permissions for IPMI device access
- Ensure BMC is properly configured

### Permission Denied Errors
- Run with appropriate privileges (typically requires root for IPMI access)
- Check user group memberships for hardware access

### High CPU Usage
- Increase monitoring interval with `--interval`
- Check for sensor reading issues causing rapid polling

## Configuration Tips

- **Power Curve**: Lower values (1.0-2.0) create gentler curves, higher values (4.0+) create steeper responses
- **Temperature Range**: Adjust min/max temps based on your CPU's specifications and cooling capacity
- **Fan Range**: Set based on your fan capabilities and noise tolerance
- **Interval**: 5-10 seconds is typically sufficient for most scenarios

## Safety Notes

- Always test with `--dry-run` before deploying
- Monitor system temperatures when first deploying
- Ensure fallback cooling mechanisms are in place
- This tool sets manual fan control - ensure it's running continuously or fans may not respond to temperature changes

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.