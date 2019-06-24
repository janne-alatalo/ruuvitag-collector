# Ruuvitag collector

This is sensor measurement collector for [ruuvitag](https://ruuvi.com/)
bluetooth sensor. It uses bluez stack through d-bus, meaning it works only on
linux. The program can print the measurements to stdout or send them to
influxdb.

# Installation

## Install the Rust programming language

https://www.rust-lang.org/tools/install

## Installation for Rasperry pi 3 on Raspbian (default OS)

Install the build dependencies.

```
sudo apt install libdbus-1-dev libssl-dev
```

This should be enough for raspbian. Jump to the *Download and Compile the
software* part.

## Installation for Rasperry pi 3 on Arch Linux

Setup bluetooth.

```
sudo pacman -S bluez bluez-utils
```

Install the bluetooth raspi packages from aur

```
mkdir aur
cd aur
git clone https://aur.archlinux.org/pi-bluetooth.git
cd pi-bluetooth
makepkg -si
```

Not 100% sure if this package is needed:

```
git clone https://aur.archlinux.org/hciattach-rpi3.git
cd hciattach-rpi3
makepkg -si
```

Override bluetooth service bluetooth service file:

```
# /etc/systemd/system/bluetooth.service.d/override.conf

[Service]
ExecStart=
ExecStart=/usr/lib/bluetooth/bluetoothd --experimental
```

Reload daemon:

```
sudo systemctl daemon-reload
```

Enable services:

```
sudo systemctl enable brcm43438.service
sudo systemctl enable bluetooth.service

sudo systemctl start brcm43438.service
sudo systemctl start bluetooth.service
```

## Download and Compile the software

Create download directory and clone the project.

```
sudo mkdir -p /opt/ruuvitag-collector
cd /opt/ruuvitag-collector
sudo chown $USER .
git clone https://github.com/janne-alatalo/ruuvitag-collector.git .
cargo build --release
```

# If you are using influxdb consumer

## Create the database and the database user

On influxdb cli, run the following commands:

```
CREATE DATABASE ruuvitag;
CREATE USER ruuvitag WITH PASSWORD 'some_secret_password';
GRANT ALL ON "ruuvitag" to "ruuvitag";

# If you are using grafana and have a user for grafana
GRANT READ ON "ruuvitag" to "grafana";
```

If you want to use data downsampling and old data deletion, run the following
commands in the influxdb cli. The following keeps the high accuracy
measurements for two weeks, but also downsamples the measurements to 5 minute
mean values and keeps them forever.

```
CREATE RETENTION POLICY "two_weeks" on "ruuvitag" DURATION 2w REPLICATION 1 DEFAULT;
CREATE RETENTION POLICY "forever" on "ruuvitag" DURATION 0s REPLICATION 1;

# For some reason influxdb cli does not support multiline queries. It is
# written on one line here so that it is easy to copy
CREATE CONTINUOUS QUERY "downsample_ruuvitag" ON "ruuvitag" BEGIN SELECT mean(*) INTO "forever"."ruuvitag" FROM "two_weeks"."ruuvitag" GROUP BY time(5m),"tag" END
```

# Configure the software

Copy the unit file form the repository.

```
sudo cp /opt/ruuvitag-collector/ruuvitag-collector.service /etc/systemd/system/ruuvitag-collector.service
```

Then create a devicemap file to `/etc/ruuvitag-collector/devicemap.json`.

```
# example of devicemap.json file
{
	"ED:11:48:07:0C:9A": {
		"tag": "ruuvi1"
	},
	"EE:23:E4:E4:E9:9C": {
		"tag": "ruuvi2"
	}
}
```

Then create `/etc/default/ruuvitag-collector`, that has the following content:

```
INFLUXDB_DB=ruuvitag
INFLUXDB_URL=http://10.8.0.1:8086

# This password is obviously the one that you set when creating the user for
# the database
INFLUXDB_PASSWORD=some_secret_password
INFLUXDB_USER=ruuvitag
RUST_LOG=info
```

Start and enable the service.

```
sudo systemctl start ruuvitag-collector
sudo systemctl enable ruuvitag-collector
```

# Problems

The following error might appear when trying to run the program:

```
bt_sensor: D-Bus error: Method "Get" with signature "ss" on interface "org.freedesktop.DBus.Properties" doesn't exist
```

In that case, try to change the `btdevice` option. Example:

```
bt-sensor --btdevice hci1
```
