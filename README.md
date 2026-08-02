# Wallie

A (planned to be) powerful wallpaper manager for the Linux Desktop. It does not render wallpapers by itself, but allows to manage automaticly switching wallpepers based on different rules and events.

IMPORTANT: Wallie is expected to work only on Linux, at least for now. It is not tested on any other *nix systems, but may work and any test information is welcome. I will never try to port it for Windows, due to how different the systems work, althrough it may become compatible "accidentaly", and contributions are welcome.

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

Start the daemon with
```sh
wallie-daemon ~/path/to/wallpaper/directory
```
This will parse the directory recursively for **all** files, including non-picture ones. Curently Wallie supports only awww as the backend and by default will change the picture every 300 seconds. The duration can be changed with the -d flag specified in integer seconds.
the ```wallie``` command can be used for the following:
```sh
wallie next #randomly chooses the next wallpaper and resets the timer
wallie reload #reparses the wallpaper directory without affecting anything else
wallie kill #kills the daemon
```

## Roadmap/Planned Features

- Rule based timed random wallpapers
- Event driven immidiate wallpaper changes
- GUI configuration

### Planned official support of wallpaper renderers
- awww
- swaybg
- mpvpaper
- Whatever KDE does

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

### AI/LLM Usage
No AI/LLM was is being and will ever be used in this project. Any pull requests with major AI/LLM written code will be rejected. Sensible bug reports written/made with the help of an AI/LLM will be taken seriously, but will not get priority, unless they are actualy important.
Use of AI/LLM for translation purposes to help in communication is allowed.

## License

[MIT](https://choosealicense.com/licenses/mit/)
