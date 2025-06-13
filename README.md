#  CoX (Copilot for Xorg)

CoX (Copilot for Xorg) is an experimental project that brings a true "copilot" experience to your Linux desktop by leveraging the MPX (Multi-Pointer X) feature of the Xorg window system. CoX enables intelligent Computer Use Agents to run alongside you, interacting with your desktop as independent users—moving their own cursors, clicking, typing, and automating tasks in real time, right next to you.

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


## TODO:

- [X] Script to create a master pointer and attach a uinput device to it
- [X] Add keyboard support
- [X] Basic CU agent loop script (full screen)
- [X] Single window support
- [ ] Refactor code for better maintainability
- [ ] Beautiful UI (window selection + prompt)

