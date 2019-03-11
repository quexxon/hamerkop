use hamerkop::ips::IPS;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        4 => {
            let patch_file = Path::new(&args[1]);
            let input_file = Path::new(&args[2]);
            let output_file = Path::new(&args[3]);

            IPS::parse(patch_file).and_then(|patches| {
                patches.apply(input_file, output_file)
            }).unwrap();
        }

        _ => {
            println!("USAGE:");
            println!("hamerkop PATCH_FILE INPUT_FILE OUTPUT_FILE");
        }
    }
}
