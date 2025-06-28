# Quiet Server

A Rust-based fan controller that automatically adjusts server fan speeds based on CPU package temperatures using IPMI and hardware sensors.

## Overview

This tool continuously monitors CPU package temperatures via lm-sensors and automatically adjusts fan speeds using IPMI commands. **The primary purpose is to make production servers quieter for home lab deployments** by providing intelligent fan curve control with configurable temperature thresholds and power curves to balance cooling performance with noise levels.

Production servers typically run fans at high speeds for maximum cooling in data center environments. This service allows you to run them more quietly at home while maintaining safe operating temperatures.

## Features

- **Automatic Temperature Monitoring**: Reads CPU package temperatures from coretemp sensors
- **Intelligent Fan Curves**: Configurable power curve with min/max temperature and fan speed settings
- **IPMI Integration**: Direct fan control via ipmitool commands
- **Continuous Operation**: Runs as a daemon with configurable monitoring intervals
- **Verbose Logging**: Optional detailed output showing temperature readings and fan adjustments
- **Dry Run Mode**: Test configuration without actually changing fan speeds

## System Requirements

### Hardware
- Server with IPMI-compatible BMC (Baseboard Management Controller)
- CPU with coretemp temperature sensors
- Linux-based operating system

### Software Dependencies
- **lm-sensors**: Hardware monitoring library and utilities
- **ipmitool**: IPMI management utility
- **libsensors-dev**: Development headers for lm-sensors
- **clang**: Required for building lm-sensors Rust bindings

### Rust Dependencies
- Rust 2024 edition or later
- See `Cargo.toml` for specific crate dependencies

## Installation

### 1. Install System Dependencies

#### Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install lm-sensors lm-sensors-dev ipmitool libclang-dev
```

#### RHEL/CentOS/Fedora:
```bash
sudo dnf install lm_sensors lm_sensors-devel ipmitool clang-devel
# or for older versions:
sudo yum install lm_sensors lm_sensors-devel ipmitool clang-devel
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
Ensure your user has access to IPMI commands:
```bash
# Test IPMI access
ipmitool chassis status

# If needed, add user to appropriate groups
sudo usermod -a -G dialout $USER
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
Create `/etc/systemd/system/quiet-server.service`:
```ini
[Unit]
Description=Quiet Server Fan Control Service
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/local/bin/quiet-server --min-fan 10 --max-fan 80
Restart=always
RestartSec=5
User=root
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Enable and start the service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable quiet-server
sudo systemctl start quiet-server

# Check status
sudo systemctl status quiet-server
sudo journalctl -u quiet-server -f
```

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

[Add your license information here]