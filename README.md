# GatorLink Bypass Codes

Proprietary 2FA app they want you to use is proprietary malware, and the phone call
authentication is very inconvenient.
UF simply refuses to adopt standard TOTP based 2FA.
That's where the bypass codes come in: you get 5 singe-use codes that replace 2FA.
Every time you generate 5 more any unused codes become invalid, so you can't just
stock up on them.
Since GatorLink really likes to log you out multiple times a day, it becomes very
inconvenient to manage the codes.
This program manages the codees for you by regenerating them for you, making sure
they never run out.

You can ask UF Help Desk at the Hub to enable 2FA bypass codes.
Make sure to bring your State ID or driver's license.

## Setup

```bash
cd gatorlink_bypass
mkdir -p .secrets
chmod 700 .secrets
touch .secrets/{password.txt,username.txt,codes.txt}
chmod 600 .secrets/*
```
