# GatorLink Bypass Codes

The 2FA app UF makes you use is proprietary malware, and the alternative phone call
authentication is very inconvenient (I usually don't carry my phone with me).
UF simply refuses to adopt standard TOTP based 2FA that works on any device.
However, UF provides a silly bypass codes system for people with phone issues:
you get 5 singe-use codes that replace 2FA.
Every time you generate 5 more, any unused codes become invalid, so you can't just
stock up on them.
Since GatorLink really likes to log you out multiple times a day, it becomes very
inconvenient to manage the codes, especially when you need them for HiPerGator work
-- they run out very fast.

This program helps you manage the codes regenerating them for you, making sure they
never run out.

You can ask UF Help Desk at the Hub to enable 2FA bypass codes.
Make sure to bring your State ID or driver's license.

## Setup

Install [geckodriver](https://github.com/mozilla/geckodriver/releases).

> **_NOTE:_**  you can use `selenium-manager` to get geckodriver:
`sudo dnf install selenium-manager` and then `selenium-manager --browser firefox`

Also you need `wl-clipboard` to use with Wayland: `sudo dnf install wl-clipboard`

```bash
cd gatorlink_bypass
mkdir -p .secrets
chmod 700 .secrets
touch .secrets/{username.txt,password.txt,codes.txt}
chmod 600 .secrets/*
```

Put your username, password, and codes into the corresponding files.

```bash
cargo build --release
mkdir -p ~/bin
cp target/release/gatorlink_bypass ~/bin
~/bin/gatorlink_bypass ~/.cache/selenium/geckodriver/linux-arm64/0.36.0/geckodriver
```
