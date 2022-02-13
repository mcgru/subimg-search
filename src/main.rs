use quicli::prelude::*;
use structopt::StructOpt;

// use image::{GenericImageView, RgbImage, imageops};
use image::{GenericImageView, RgbImage};


///
/// Standard quicli interface to command line args
///
#[derive(StructOpt, Debug)]
#[structopt(name = "subimg-search")]
struct Cli {
    /// The target image where to search subimage
    image: String,

    /// The target subimage what to search in image
    subimage: String,

    #[structopt(flatten)]
    verbosity: Verbosity,

    /// How many pixels from border to skip?
    #[structopt(short = "b", default_value = "0")]
    skip_border: u32,

    /// Output fodler to put result in
    #[structopt(short = "d", default_value ="")]
    output_folder: String,

    /// Output result into this file
    #[structopt(short = "o", default_value ="")]
    output: String,

}

///
/// Program subimg-search searches a subimage in the image and prints the most-probability (RSME) b/w-map
///
fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger(&env!("CARGO_PKG_NAME"))?;

    info!("Reading {:?}...", args.subimage);
    let sub = read_image(&args.subimage)?;

    info!("Reading {:?}...", args.image);
    let img = read_image(&args.image)?;

    assert!( img.width()  > sub.width() +args.skip_border*2 );
    assert!( img.height() > sub.height()+args.skip_border*2 );

    info!("Starts search subimage in image...");
    do_search(&img, &sub, &args)?;

    use std::env;
    info!("OK: {:?} finished successfully", env::args());

    Ok(())
}

fn read_image(thefilename: &String) -> image::ImageResult<image::DynamicImage> {
    let img = image::open(std::path::PathBuf::from(thefilename).as_path())?;
    Ok(img)
}


use bash_like_utils::PathString;

/// Calculate new name for result file from input name and output folder if any
fn calc_new_name( args: &Cli) -> Result<String,Error> {
    let fname = 
    if args.output.is_empty() {
        // need to create if empty
        let outdir =
        if args.output_folder.is_empty() {
            // use folder of input file
            PathString::dirname(PathString::readlink(&args.image).as_str())
        }else {
            format!("{}",args.output_folder)
        };
        let bname = format!("{}.MAP.{}.{}", PathString::basenoext(&args.image), PathString::basenoext(&args.subimage), PathString::extension(&args.image) );
        format!("{}/{}", outdir, bname )
    } else {
        // just use it
        args.output.clone()
    };
 Ok(fname)
}



/// quicker version - use only diagonal pixels
// pub fn calc_root_error_squares_mean1( sub : &image::RgbImage, sam: image::SubImage<&RgbImage> ) -> f32 {
//     // let sub = sub.as_rgb8().unwrap();
//     // let sam = sam.as_rgb8().unwrap();
//     let (i1w, i1h) = sub.dimensions();
//     let (i2w, i2h) = sub.dimensions();
//     let width  = if i1w<=i2w { i1w } else { i2w } ;
//     let height = if i1h<=i2h { i1h } else { i2h } ;
//     let minsize= if width<=height { width } else { height } ;
//     let qty = minsize;
//     let mut acc = 0_f32;
//     // let mut add = 0f64;
//     // let mut dbgstr : String;
//     for i in 0 .. minsize {
//         let i1p = sub[(i,i)];
//         // let i2p = sam.inner().get_pixel(i,i);
//         let i2p = sam.get_pixel(i,i);
//         let i1p = [ i1p[0] as i32, i1p[1] as i32, i1p[2] as i32 ];
//         let i2p = [ i2p[0] as i32, i2p[1] as i32, i2p[2] as i32 ] ;
//            // dbg!((i,i,i1p,i2p));
//         // acc += ((i1p[0]-i2p[0]) as f32) * ((i1p[0]-i2p[0]) as f32);
//         // acc += ((i1p[1]-i2p[1]) as f32) * ((i1p[1]-i2p[1]) as f32);
//         // acc += ((i1p[2]-i2p[2]) as f32) * ((i1p[2]-i2p[2]) as f32);
//         // add  = ((i1p[0]-i2p[0]) * (i1p[0]-i2p[0])) as f32;
//         acc += ((i1p[0]-i2p[0]) * (i1p[0]-i2p[0])) as f32;
//         acc += ((i1p[1]-i2p[1]) * (i1p[1]-i2p[1])) as f32;
//         acc += ((i1p[2]-i2p[2]) * (i1p[2]-i2p[2])) as f32;
//           // dbgstr = format!("{:?}",(i,i,i1p,i2p,add,acc));
//           // dbg!(dbgstr);
//     };
//     ( acc / (qty as f32) / 3.0 ).powf(0.5) as f32
// }

/// full version - calc RMSE on all pixels in sample
pub fn calc_root_error_squares_mean_full( sub : &image::RgbImage, sam: image::SubImage<&RgbImage> ) -> f32 {
    // let sub = sub.as_rgb8().unwrap();
    // let sam = sam.as_rgb8().unwrap();
    let (i1w, i1h) = sub.dimensions();
//    let (i2w, i2h) = sub.dimensions();
//    let width  = if i1w<=i2w { i1w } else { i2w } ;
//    let height = if i1h<=i2h { i1h } else { i2h } ;
//    let minsize= if width<=height { width } else { height } ;
    let qty = i1w; // i1w*i1h for nice picture as in ImageMagick::compare -subimage-search -metric RMSE
    let mut acc = 0_f32;
    for y in 0 .. i1h {
    for x in 0 .. i1w {
        let i1p = sub[(x,y)];
        if i1p[0] == 0 && i1p[1] == 0 && i1p[2] == 0 {
            // skip black pixel in subimage
            continue;
        }
        // let i2p = sam.inner().get_pixel(i,i);
        let i2p = sam.get_pixel(x,y);
        let i1p = [ i1p[0] as i32, i1p[1] as i32, i1p[2] as i32 ];
        let i2p = [ i2p[0] as i32, i2p[1] as i32, i2p[2] as i32 ] ;
           // dbg!((i,i,i1p,i2p));
        // acc += ((i1p[0]-i2p[0]) as f32) * ((i1p[0]-i2p[0]) as f32);
        // acc += ((i1p[1]-i2p[1]) as f32) * ((i1p[1]-i2p[1]) as f32);
        // acc += ((i1p[2]-i2p[2]) as f32) * ((i1p[2]-i2p[2]) as f32);
        // add  = ((i1p[0]-i2p[0]) * (i1p[0]-i2p[0])) as f32;
        acc += ((i1p[0]-i2p[0]) * (i1p[0]-i2p[0])) as f32;
        acc += ((i1p[1]-i2p[1]) * (i1p[1]-i2p[1])) as f32;
        acc += ((i1p[2]-i2p[2]) * (i1p[2]-i2p[2])) as f32;
          // dbgstr = format!("{:?}",(i,i,i1p,i2p,add,acc));
          // dbg!(dbgstr);
    }};
    ( acc / (qty as f32) / 3.0 ).powf(0.5) as f32
}

// fn do_map_at_pixel(img: &image::RgbImage, x: u32, y: u32, sub: &image::RgbImage, sw: u32, sh: u32, res: &mut RgbImage) -> Result<(), Error> {
//     let sam = img.view( x, y, sw, sh);
//     let pxerr = 255 - (calc_root_error_squares_mean_full(sub, sam) as u8);
//     res.put_pixel( x+sw/2, y+sh/2, image::Rgb([pxerr,pxerr,pxerr]) );
// Ok(())
// }

// fn do_map_at(img: &image::RgbImage, x: u32, y: u32, sub: &image::RgbImage, sw: u32, sh: u32) -> u8 {
//     let sam = img.view( x, y, sw, sh);
//     255 - (calc_root_error_squares_mean_full(sub, sam) as u8)
// }

use switch_statement::switch;

fn do_search(img: &image::DynamicImage, sub: &image::DynamicImage, args: &Cli) -> Result<(), Error> {

    let fname = calc_new_name(&args).unwrap();

    let img = img.as_rgb8().unwrap();
    let sub = sub.as_rgb8().unwrap();

    let iw = img.width()  - args.skip_border*2;
    let ih = img.height() - args.skip_border*2;
    let sw = sub.width();
    let sh = sub.height();

// thrqty: number of threads = number of strips
// subqty: how much full subimage's heights fit in image height
// pih: part of image height = image height divided by thrqty :: approximately
// piw: part of image width  = same as full image width
// ty: thread's strip's starting y-coord
// th: thread's strip's height
// 
    let subqty = (ih-2*args.skip_border)/sh ; // ih>=sh
    let mut thrqty = num_cpus::get() as u32 ;
    if subqty < thrqty { thrqty=subqty };
    if subqty < 4      { thrqty=1 };

    let ( piw, pih ) = ( iw-2*args.skip_border, (ih-2*args.skip_border) / thrqty );

    // use std::time::{Duration, Instant};
    // let start = Instant::now();

    let mut img_final = image::RgbImage::new( iw, ih );

    if thrqty >= 2 {

        info!("multi-thread case with {} threads",thrqty);

        let results : Vec<(u32, image::RgbImage)> =
        (0..thrqty).into_par_iter().map( move |t| {

          let ty = t * pih + args.skip_border;

          let th = switch! { t;
            thrqty-1 => (ih-args.skip_border)-ty,
            _ =>  pih+sh,
          };

          let img_view = img.view( args.skip_border, ty, piw, th );

          let mut img_mt_accum = image::RgbImage::new( piw, th );

          for y in    0 .. th-sh  { // each row
             for x in 0 .. piw-sw { // each pixel
               let sam = img_view.view( x, y, sw, sh);
               let pxerr = 255 - (calc_root_error_squares_mean_full(sub, sam) as u8);
               img_mt_accum.put_pixel( x+sw/2, y+sh/2, image::Rgb([pxerr,pxerr,pxerr]) );
             };
          };
          (ty, img_mt_accum)
        }).collect();

        info!("multi-thread case: done with threads, going to glue results");

        for (y, r) in results {
           // imageops::overlay(&mut img_final, &r, args.skip_border as i64, y as i64);
            for yi in 0 .. r.height() {
                for xi in 0 .. r.width() {
                    let pover = r.get_pixel(xi, yi);
                    let pbase =   img_final.get_pixel(xi+args.skip_border, y + yi );  // y already has +args.skip_border
                    let psumm = (pbase[0] as u32) + (pover[0] as u32);
                    let psumm = if psumm <= 255 { psumm } else { 255 } as u8;
                    img_final.put_pixel( xi+args.skip_border, y + yi, image::Rgb([psumm,psumm,psumm]) );   // y already has +args.skip_border
                }
            }
        }

    } else {

        info!("single-thread case");

        for y in     args.skip_border .. ih-sh { // each row
            for x in args.skip_border .. iw-sw-2*args.skip_border { // each pixel
               let sam = img.view( x, y, sw, sh);
               let pxerr = 255 - (calc_root_error_squares_mean_full(sub, sam) as u8);
               img_final.put_pixel( x+sw/2, y+sh/2, image::Rgb([pxerr,pxerr,pxerr]) );
            };
        };
    }

// let duration = start.elapsed();
// eprintln!("Time elapsed in expensive_function() is: {:?}", duration);


    img_final.save(fname).unwrap();

Ok(())
}
