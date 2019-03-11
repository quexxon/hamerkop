use ips_patcher::IpsRecord;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        4 => {
            let patch_file = Path::new(&args[1]);
            let input_file = Path::new(&args[2]);
            let output_file = Path::new(&args[3]);

            let file = fs::read(&patch_file).unwrap();
            let eof_start = file.len() - 3;

            let mut buffer = fs::read(&input_file).unwrap();

            assert_eq!(&file[0..5], b"PATCH");
            assert_eq!(&file[eof_start..file.len()], b"EOF");

            let mut index = 5;
            while index < eof_start {
                let (record, next_index) = IpsRecord::new(&file, index);
                index = next_index;
                record.apply_patch(&mut buffer);
            }

            fs::write(&output_file, &buffer).unwrap();
        }

        _ => {
            println!("USAGE:");
            println!("cmd PATCH_FILE INPUT_FILE OUTPUT_FILE");
        }
    }
}
