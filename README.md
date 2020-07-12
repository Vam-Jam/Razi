<h2 align = 'center'><img src="https://cdn.discordapp.com/attachments/386495658418503680/731537691405189150/hackerman.jpg"><br>Razi, the cat bot</h2>

<p align = 'center'> Currently being developed.</p>

# Why ?
Razi is a bot made to replace Riza. You can find riza's source code [here - Click me](https://github.com/Vam-Jam/Riza_).
The main issue with riza was poor error handling, timing out and much more. To fix this, i wanted to start over from scratch and make it far more easier to use and maintain.

Razi is using [serenity-rs api](https://github.com/serenity-rs/serenity)

# How did you make it easier to 'maintain' ?
The main way was too add a razi's toml file, which lets you change the token, prefixes, allowed channels, owners.
```
[discord]
token = ""
prefixes = [".", "~"]
allowed_channels = [386495658418503680, 610155351047667752]
owners = [233738372814864395, 301370472594014218, 165600688590553088, 142300409002852352]
```
<br>

Along with this, you also have a kag server config, which allows you to add a new server for server_request command.
<br>

```
[[kag_server]]
names = ["mc", "3d", "goldenguy"]
ip = "85.10.195.233"
port = "50303"
minimap = true
```

# Can I use it in my server?
Go for it! Source code is here for anybody to use.<br>
I'll add a guide on how to do it later on, feel free to message me if you dont understand how to set it up.
<br>

# Thank you notes:
- Thank you to the serenity-rs team, they have helped me fix issues in the past.=
- Caesar for making the picture
