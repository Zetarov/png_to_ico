use std::{env};
use std::path::Path;
use image::{GenericImageView, imageops};
use progress_bar::progress_bar::ProgressBar;
use progress_bar::color::{Color, Style};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("This program generates resized images of the original image, it will be generated in the same folder with the name \"original_image_nameXXX.png\" where XXX is the size of the image (width/height). A last image with name \"original_image_name_ico.ico\" will be generated containing all resolutions from the others.\n");
        println!("Syntax: {} path_to_image.png 16 24 32 48 64 96 128 192 256...\n", args[0]);
        return;
    }

    let image_sizes = if args.len() == 2 {
        println!("The image will be resized to default dimensions.");
        vec![16, 24, 32, 48, 64, 96, 128, 192, 256]
    }
    else {
        let mut sizes: Vec<u32> = Vec::new();
        for arg in args.iter().skip(2) {
            let dimension: u32 = arg.parse().expect(&format!("Expected number not \"{}\"", arg));
            if dimension > 256 {
                eprintln!("Resize dimension should not be higher than 256, received \"{}\"", dimension);
                std::process::exit(-1);
            }
            sizes.push(dimension);
        }
        sizes
    };

    let image_path = String::from(args[1].as_str()); 
    let image_path_without_ext = Path::new(&image_path).file_stem().unwrap();
    let extension = Path::new(&image_path).extension().unwrap();
    let mut parent_dir = Path::new(&image_path).to_path_buf();
    parent_dir.pop();

    let image = image::open(&image_path).unwrap();
    let buffered_image = image.to_rgba8();

    if image.dimensions().0 != image.dimensions().1 {
        println!("Image is not a square image. Aborting.");
        return;
    }

    let mut png_pathes: Vec<Box<Path>> = Vec::new();

    let mut resize_progress_bar = ProgressBar::new(image_sizes.len());
    resize_progress_bar.set_action("Resizing", Color::White, Style::Bold);
    for image_target_size in image_sizes {
        resize_progress_bar.print_info("Resizing", &format!("to {}... ", image_target_size), Color::White, Style::Normal);
        if image_target_size > image.dimensions().0 {
            println!("\t🟡 Skipped (resize dimension is greater than image dimensions {} > {})", image_target_size, image.dimensions().0);
            resize_progress_bar.inc();
            resize_progress_bar.print_info("Skipped", &format!("(resize dimension is greater than image dimensions {} > {})", image_target_size, image.dimensions().0), Color::LightYellow, Style::Normal);
            continue;
        }
        let resized = imageops::resize(&buffered_image, image_target_size, image_target_size, imageops::FilterType::Gaussian);
        let mut save_path = parent_dir.clone();
        save_path.push(format!("{}{}.{}", image_path_without_ext.to_str().unwrap(), image_target_size, extension.to_str().unwrap()));
        resized.save_with_format(save_path.clone(), image::ImageFormat::Png).unwrap();
        png_pathes.push(save_path.into_boxed_path());
        resize_progress_bar.inc();
        resize_progress_bar.print_info("Done.", "", Color::LightGreen, Style::Normal);
    }
    resize_progress_bar.finalize();

    let mut packing_progress_bar = ProgressBar::new(png_pathes.len());
    print!("Packing this into .ico... ");
    packing_progress_bar.set_action("Packing", Color::White, Style::Bold);
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for png_path in png_pathes.iter() {
        let file = std::fs::File::open(png_path).unwrap();
        let image = ico::IconImage::read_png(file).unwrap();
        icon_dir.add_entry(ico::IconDirEntry::encode(&image).unwrap());
        packing_progress_bar.inc();
    }
    packing_progress_bar.finalize();

    let mut delete_progress_bar = ProgressBar::new(png_pathes.len());
    delete_progress_bar.set_action("Cleaning", Color::White, Style::Bold);
    for png_path in png_pathes.iter() {
        std::fs::remove_file(png_path).unwrap();
        delete_progress_bar.inc();
    }
    delete_progress_bar.finalize();


    let mut save_path = parent_dir.clone();
    save_path.push(format!("{}_ico.ico", image_path_without_ext.to_str().unwrap()));
    let file = std::fs::File::create(save_path).unwrap();
    icon_dir.write(file).unwrap();
}