//#![feature(trusted_len)]
//#![feature(test)]
extern crate compt;
extern crate axgeom;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree;
extern crate rayon;
extern crate duckduckgeo;
extern crate dists;
extern crate gnuplot;
//extern crate test;

pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

mod inner_prelude{
    pub(crate) use crate::FigureBuilder;
    pub use crate::support::*;
    pub use dinotree_alg::colfind;
    pub use dinotree::*;
    pub use dinotree::advanced::*;
    pub(crate) use axgeom;
    pub(crate) use crate::datanum;
    pub use gnuplot::*;
    pub(crate) use dists;
    pub use std::time::Instant;
    pub use std::time::Duration;
    pub use crate::black_box;
    pub use num_traits::cast::AsPrimitive;
}

#[macro_use]
mod support;
mod colfind;
mod spiral;
pub(crate) mod datanum;

use gnuplot::*;
use std::env;

pub struct FigureBuilder{
    folder:String,
    last_file_name:Option<String>
}

impl FigureBuilder{
    fn new(folder:String)->FigureBuilder{
        FigureBuilder{folder,last_file_name:None}
    }
    fn build(&mut self,filename:&str)->Figure{
        let mut fg = Figure::new();
        let ss=format!("{}/{}.gplot",&self.folder,filename);
        //println!("Creating {}",ss);
        
        //let ss2=format!("{}/{}.png",&self.folder,filename);
        fg.set_terminal("pngcairo size 800,600 enhanced font 'Veranda,10'","");
        
        fg.set_pre_commands("set output system(\"echo $FILE_PATH\")");        

        //set terminal pngcairo size 350,262 enhanced font 'Verdana,10'
        self.last_file_name=Some(ss);
        //fg.set_terminal("pngcairo",&ss);// size 1024, 800
        fg
    }
    fn finish(&mut self,figure:Figure){
        figure.echo_to_file(&self.last_file_name.take().unwrap());
        //figure.show();
    }
}


use std::path::Path;
use std::io::Write;
use std::time::*;


fn into_secs(elapsed: std::time::Duration) -> f64 {
    (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0)
}

// This is a simple macro named `say_hello`.
macro_rules! run_test {
    // `()` indicates that the macro takes no argument.
    ($fo:expr,$tre:expr) => (
        // The macro will expand into the contents of this block.
        print!("Running {}...",stringify!($tre));
        std::io::stdout().flush().unwrap();
        let time=Instant::now();
        $tre($fo);
        let val=into_secs(time.elapsed());
        println!("finished in {} seconds.",val);

    )
}


fn main() {

    //to run program to generate android bench data.
    //build armv7-linux-androideabi
    //adb -d push dinotree_alg_data /data/local/tmp/dinotree_data
    //adb -d shell pm grant /data/local/tmp/dinotree_data android.permission.WRITE_EXTERNAL_STORAGE
    //adb -d shell /data/local/tmp/dinotree_data bench /sdcard/dinotree/graphs
    //adb -d pull "/sdcard/dinotree/graphs"
    //
    //TODO
    //seperate into benches versus theory runs
    //run benches on laptop/new gaming laptop/android phone/web assembly, and compare differences.
    // 

    //println!("{:?}",stringify!(spiral::handle));
            
    let args:Vec<String> = env::args().collect();
    //assert_eq!(args.len(),2,"First arguent needs to be gen or graph");

    match args[1].as_ref(){
        "theory"=>{
            let folder=args[2].clone();
            let path=Path::new(folder.trim_end_matches('/'));
            std::fs::create_dir_all(&path).expect("failed to create directory");
            let mut fb=FigureBuilder::new(folder);
            
            run_test!(&mut fb,spiral::handle);
            run_test!(&mut fb,colfind::colfind::handle_theory);
            run_test!(&mut fb,colfind::construction_vs_query::handle_theory);
            run_test!(&mut fb,colfind::level_analysis::handle_theory);
            run_test!(&mut fb,colfind::theory_colfind_3d::handle);
        }
        "bench"=>{
            let folder=args[2].clone();
            let path=Path::new(folder.trim_end_matches('/'));
            std::fs::create_dir_all(&path).expect("failed to create directory");
            let mut fb=FigureBuilder::new(folder);
            
            run_test!(&mut fb,colfind::colfind::handle_bench);
            run_test!(&mut fb,colfind::copy_vs_nocopy::handle);
            run_test!(&mut fb,colfind::construction_vs_query::handle_bench);
            run_test!(&mut fb,colfind::rebal_strat::handle);
            run_test!(&mut fb,colfind::float_vs_integer::handle);
            run_test!(&mut fb,colfind::level_analysis::handle_bench);
            run_test!(&mut fb,colfind::parallel_heur_comparison::handle);
            
            //This is the one thats interesting to see what the results are on phone/vs/laptop
            run_test!(&mut fb,colfind::height_heur_comparison::handle);
            

            //nbody::theory::handle(&mut fb);
        },
        "graph"=>{
            let folder=args[2].clone();

            let path=Path::new(folder.trim_end_matches('/'));


            let target_folder=args[3].clone();
            let target_dir=Path::new(target_folder.trim_end_matches('/'));
            std::fs::create_dir_all(&target_dir).expect("failed to create directory");


            //println!("path={:?}",path);
            //println!("target dir={:?}",target_dir);


            let paths = std::fs::read_dir(path).unwrap();

            for path in paths {
                let path=match path{
                    Ok(path)=>path,
                    _=>continue
                };



                if let Some(ext) = path.path().extension(){
                    if ext == "gplot"{
                        let path_command=path.path();
                        println!("generating {:?}",path.file_name());

                        
                        //let output=format!("-e \"output='{}' \"",path.path().with_extension("png").to_str().unwrap());
                        //gnuplot -e "filename='foo.data'" foo.plg
                    
                        let mut command=std::process::Command::new("gnuplot");

                        //println!("filename={:?}",path.path().file_stem().unwrap());


                        let new_path=path.path().with_extension("png");
                        let blag=Path::new(new_path.file_name().unwrap().to_str().unwrap());
                        let file_path=target_dir.join(blag);
                        command
                            .arg("-p")
                            .arg(path_command)
                            .env("FILE_PATH",file_path.to_str().unwrap());

                        //println!("command={:?}",command);

                        command.status()
                            .expect("Couldn't spawn gnuplot. Make sure it is installed and available in PATH.");
                    }
                }
            }
            //gnuplot -p "colfind_rebal_vs_query_num_bots_grow_of_1.gplot"
            println!("Finished generating graphs");
        },
        _=>{
            println!("First argument must be gen or graph");
        }
    }

    

}
