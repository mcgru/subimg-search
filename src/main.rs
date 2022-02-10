use quicli::prelude::*;
use structopt::StructOpt;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use image::{GenericImageView, ImageBuffer, RgbImage, imageops};


///
/// Program subimg-search searches a subimage in the image and prints the most-probability (RSME) b/w-map
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
    #[structopt(short = "s", default_value = "100")]
    skip_border: u32,

    /// Output fodler to put result in
    #[structopt(short = "d", default_value ="")]
    output_folder: String,

    /// Output result into this file
    #[structopt(short = "o", default_value ="")]
    output: String,

}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger(&env!("CARGO_PKG_NAME"))?;

    // dbg!(args);
    info!("Reading {:?}...", args.subimage);
    let sub = read_image(&args.subimage)?;

    info!("Reading {:?}...", args.image);
    let img = read_image(&args.image)?;

    //   info!("Checking {:?} for appropriate properties...", args.target);
    // check_image(&img)?;

    info!("Starts search subimage in image...");
    do_search(&img, &sub, &args)?;

    use std::env;
    info!("OK: {:?} finished successfully", env::args());

    Ok(())
}

// fn read_image(thefilename: &String) -> Result<image::DynamicImage,Error> {
fn read_image(thefilename: &String) -> image::ImageResult<image::DynamicImage> {
    let img = image::open(std::path::PathBuf::from(thefilename).as_path())?;
    Ok(img)
}


#[derive(Debug, Clone)]
pub struct Papath;

impl Papath {
    pub fn readlink(path: &str) -> String {
        let path = Path::new(path).canonicalize().unwrap_or(PathBuf::new());
        path.to_str().unwrap_or("").to_string()
    }
    pub fn dirname(path: &str) -> String {
        let path = Path::new(path).parent().unwrap_or(Path::new(""));
        path.to_str().unwrap_or("").to_string()
    }
    pub fn basename(path: &str) -> String {
        let path = Path::new(path).file_name().unwrap_or(OsStr::new(""));
        path.to_str().unwrap_or("").to_string()
    }
    pub fn extension(path: &str) -> String {
        let bn = Papath::basename(path);
        let bn : Vec<_> = bn.split(".").collect();
        // dbg!(bn.len());
        match bn.len() {   0..=1 => String::new(),  _ => bn.last().unwrap().to_string()   }
    }
    pub fn basenoext(path: &str) -> String {
        let bn = Papath::basename(path);
        let mut bn : Vec<_> = bn.split(".").collect();
        // dbg!(bn.len());
        match bn.len() {   0 => String::new(), 1 => { bn[0].to_string().clone() },  _ => { bn.pop(); bn.last().unwrap().to_string().clone() } }
    }
    pub fn file_stem(path: &str) -> String {
        Papath::basenoext(path)
    }
}


fn calc_new_name( args: &Cli) -> Result<String,Error> {
    // calculate new name for result file
    let fname = 
    if args.output.is_empty() {
        // need to create if empty
        let outdir =
        if args.output_folder.is_empty() {
            // use folder of input file
            Papath::dirname(Papath::readlink(&args.image).as_str())
        }else {
            format!("{}",args.output_folder)
        };
        let bname = format!("{}.MAP.{}.{}", Papath::basenoext(&args.image), Papath::basenoext(&args.subimage), Papath::extension(&args.image) );
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
    let (i2w, i2h) = sub.dimensions();
    let width  = if i1w<=i2w { i1w } else { i2w } ;
    let height = if i1h<=i2h { i1h } else { i2h } ;
    let minsize= if width<=height { width } else { height } ;
    let qty = minsize;
    let mut acc = 0_f32;
    // let mut add = 0f64;
    // let mut dbgstr : String;
    for y in 0 .. minsize {
    for x in 0 .. minsize {
        let i1p = sub[(x,y)];
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


fn do_search(img: &image::DynamicImage, sub: &image::DynamicImage, args: &Cli) -> Result<(), Error> {

    let thrqty = num_cpus::get() as u32 + 1;
    // let _ssd_pool = rayon::ThreadPoolBuilder::new().num_threads(thrqty).build().unwrap();

    if img.height() < 3 {
        bail!(format!("Image height is too small: {}",img.height()));
    }
    if img.height() < 2 {
        bail!(format!("Subimage height is too small: {}",sub.height()));
    }
    if img.height() < (sub.height()+1) {
        bail!(format!("Image height is lesser than subimage height+1: {} vs {}",img.height(), sub.height() ));
    }

    let fname = calc_new_name(&args).unwrap();

    let img = img.as_rgb8().unwrap();
    let sub = sub.as_rgb8().unwrap();

  let iw = img.width();
  let ih = img.height();
  let sw = sub.width();
  let sh = sub.height();

    let ( piw, pih ) = ( iw, ih / thrqty );

    // use std::time::{Duration, Instant};
    // let start = Instant::now();

use switch_statement::switch;        

    let results : Vec<(u32, image::RgbImage)> = 
    (0..thrqty).into_par_iter().map( move |t| {

      let py = switch! { t;
        0 => 0,
        thrqty-1 => t*pih-(sh+1),
        _ =>  t*pih,
      };
      let ph = switch! { t;
        thrqty-1 => ih-(t*pih-(sh+1)),
        _ =>  pih+sh+1,
      };

    //  dbg!((t_1,t,pih,ih,py,ph));

      let imgpart = img.view( 0, py, piw, ph );

      let mut res = image::RgbImage::new( piw, ph );

      for y in  0 .. ph-sh { // each row
         for x in 0 .. piw-sw { // each pixel
           let sam = imgpart.view( x, y, sw, sh);
           let pxerr = 255 - (calc_root_error_squares_mean_full(sub, sam) as u8);
           res.put_pixel( x+sw/2, y+sh/2, image::Rgb([pxerr,pxerr,pxerr]) );
         };
      };
      (py, res)
    }).collect();

// let duration = start.elapsed();
// eprintln!("Time elapsed in expensive_function() is: {:?}", duration);

  let mut res = image::RgbImage::new( iw, ih );

  for (y, i) in results {
    imageops::overlay(&mut res, &i, 0, y as i64);
  }


    res.save(fname).unwrap();

Ok(())
}
