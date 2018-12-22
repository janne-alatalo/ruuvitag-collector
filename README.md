# Installation for RPI3 Arch

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

The following error might appear when trying to run the program:

```
bt_sensor: D-Bus error: Method "Get" with signature "ss" on interface "org.freedesktop.DBus.Properties" doesn't exist
```

In that case, try to change the `btdevice` option. Example:

```
bt-sensor --btdevice hci1
```
