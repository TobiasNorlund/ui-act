#  CoX (Copilot for Xorg)

CoX (Copilot for Xorg) is an experimental project that brings a true "copilot" experience to your Linux desktop by leveraging the MPX (Multi-Pointer X) feature of the Xorg window system. CoX enables Computer Use agents to run alongside you, interacting with your desktop as independent users—moving their own cursors, clicking, typing, and automating tasks in real time, right next to you.

## Setup

Add this to `/etc/udev/rules.d/99-uinput.rules`:
```
KERNEL=="uinput", MODE="0660", GROUP="input"
```
Then reload:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Then add your user to `input` group:

```bash
sudo usermod -aG input $USER
newgrp input
```
