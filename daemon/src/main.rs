use std::{env::args, io::Error, path::PathBuf, process::Command, str::FromStr, thread::sleep, time::Duration};

use rand::{RngExt, distr::{Distribution, weighted::WeightedIndex}, rngs::SmallRng};

fn main() -> Result<(), Error> {
    let arguments = PassedArguments::new();
    println!("Hello, world!");
    let mut generator: SmallRng = rand::make_rng();
    let (pictures, weights) = parse_nonrecuresive(arguments.wallpaper_directory)?;
    let distribuion = WeightedIndex::new(weights.iter()).unwrap();
    loop {
        println!("{:#?}", Command::new("/bin/awww").args(["img", pictures[distribuion.sample(&mut generator)].to_str().unwrap()]).output());
        println!("{}", generator.random::<u32>());
        sleep(arguments.timing);
    }
    Ok(())
}

fn parse_nonrecuresive(directory: PathBuf) -> Result<(Vec<PathBuf>, Vec<u8>), Error> {
    let mut entries = directory.read_dir()?;
    let mut pictures = Vec::<PathBuf>::new();
    let mut distribution = Vec::<u8>::new();
    loop {
        let entry = entries.next();
        if let Some(read) = entry {
            let picture = read?.path();
            if picture.is_file() {
                pictures.push(picture);
                distribution.push(1);
            }
        } else {
            break;
        }
    }
    Ok((pictures, distribution))
}

struct PassedArguments {
    wallpaper_directory: PathBuf,
    timing: Duration,
}

impl PassedArguments {
    fn new () -> Self {
        let arguments = args().collect::<Vec<String>>();
        let mut timing = Duration::from_secs(300);
        if arguments.contains(&"-d".to_string()) {
            timing = Duration::from_secs(u64::from_str(arguments[arguments.iter().position(|entry| entry == &"-d".to_string()).unwrap()+1].as_str()).expect("Invalid sleep duration!"));
        }
        let wallpaper_directory = PathBuf::from(arguments.last().expect("No directory provided!"));
        Self { wallpaper_directory, timing }
    }
}
