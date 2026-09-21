# <span style="color: white">Wallie</span>

A (planned to be) powerful wallpaper manager for the Linux Desktop. It does not render wallpapers by itself, but allows to manage automaticly switching wallpapers based on different rules and events.

<span style="color: gold">**IMPORTANT:**</span> Wallie is expected to work only on Linux (tested on Void Linux), at least for now. It is not tested on any other *nix systems, but may work and any test information is welcome. I will never try to port it for Windows, due to how different the systems work, althrough it may become compatible "accidentaly", and contributions are welcome.

## The Idea

This wallpaper manager is created for people, who like their wallpaper change automaticaly, but are often annoyed by a white wallpaper at night. **Wallie** will use tags and rules for the pictures to be displayed at the correct time, as well as much more, like based on system state, started applications, time of year or weather.

### Implementation caveats

Not all declared here features are implemented. A GUI is not yet present, so even though Wallie is usable if you are fine with copying a lot of stuff, it is not expected to be used.

Due to how every single person has their own wallpaper gallery, you would have to define tags and rules yourself. The goal of Wallie is to be able to streamline this process to be easy and not too long.

### How does it work?

There are four important parts:

1. Wallpapers
2. Tags
3. Rules
4. Events

#### Wallpapers

Wallpapers can be sourced from one directory. Technicaly, any files can be used, but wallpaper managers may have problems with showing them. <span style="color: gold">**IMPORTANT:**</span> Wallie does not yet check for the renderer's suported filetypes.

#### Tags

Each wallpaper needs to get assigned one or more tag to be found. E. g. "dark", "anime", "christmas".

#### Rules

Rules are the central feature of Wallie. Each rule has a condition, that could be time of day, month, running apps, some system state, a script or any other thing. List of tags defines which pictures are activated. Weight allows to make the picture come up more often. Level decides on whic rule has priority in desiding th weight or can completely disable all lower rules with opcity toggle.

#### Events

Evants are rules that change the wallpaper immidiately upon becoming true.

## Installation

### Binaries
Binaries are available only for Linux (tested under Void Linux) under Releases after the first release. You also need some wallpaper renderer, supported ones are listed below, but any that support terminal control may work.

### From Source
For unsupported platforms or latest features you can compile from source directly. Make sure to have git, cargo and some shell installed.
```sh
git clone https://github.com/silverfairies/Wallie
cd Wallie
cargo build --release
```
To install into $HOME/.cargo:
```sh
cp target/release/{wallie,wallie-daemon} $HOME/.cargo/bin
```
or
```sh
cp target/release/{wallie,wallie-daemon} $HOME/bin
```

## Usage

### Daemon
Start the daemon with
```sh
wallie-daemon --simple /path/to/wallpaper/directory
```
This will parse the directory recursively for **all** files, including non-picture ones, which may lead to errors on the backend side. By default wallie will change the picture every 300 seconds. The duration can be changed with the -d flag specified in integer seconds.

### Available backends:
Can be specified with the -r flag
- awww
- swaybg
- auto (Defaults to auto, which itself defaults to awww if can not determine the current wallpaper renderer.)

### CLI Interface
Help for ```wallie``` is available:
```sh
wallie help
```

### GUI Interface
<span style="color: gold">**Not yet implemented**</span>

## Roadmap/Planned Features

- Rule based timed random wallpapers
- Event driven immidiate wallpaper changes
- GUI configuration

### Planned official support of wallpaper renderer backends
- [awww](https://codeberg.org/LGFae/awww) ✔
- [swaybg](https://github.com/swaywm/swaybg) ✔
- [mpvpaper](https://github.com/GhostNaN/mpvpaper)
- Whatever KDE-Plasma does
- COSMIC
- <span style="color: cornflowerblue">This list can be extended!</span>

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

### AI/LLM Usage
No AI/LLM was is being and will ever be used in this project. Any pull requests with major AI/LLM written code will be rejected. Sensible bug reports written/made with the help of an AI/LLM will be taken seriously, but will not get priority, unless they are actually important.
Use of AI/LLM for translation purposes to help in communication is allowed.

## License

[MIT](https://choosealicense.com/licenses/mit/)
