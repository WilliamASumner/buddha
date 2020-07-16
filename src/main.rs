extern crate image;
extern crate rand;
extern crate ndarray;

use image::{ImageBuffer, Rgb};
use rand::prelude::*; // rng
use ndarray::prelude::*; // efficient grid of values
use std::time::Instant;
use rand::seq::SliceRandom;

pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

impl Complex {
    pub fn new(r: f64, i: f64) -> Complex {
        Complex {
            real: r,
            imag: i,
        }
    }

    pub fn add(&mut self, compl: &Complex) -> &mut Self { // add
        self.real += compl.real;
        self.imag += compl.imag;
        self
    }

    pub fn addi(&self, r: f64, i: f64) -> Self { // add
        Complex {
            real: self.real + r,
            imag: self.imag + i,
        }
    }

    pub fn subi(&self, r:f64, i:f64) -> Self { // subtract immediate
        Complex {
            real: self.real - r,
            imag: self.imag - i,
        }
    }

    pub fn square(&mut self) -> &mut Self {
        // c = re + i*im, c^2 = re^2 + 2*re*im - im^2
        let tempx: f64 = self.real;
        self.real = self.real*self.real - self.imag*self.imag;
        self.imag = 2.*self.imag*tempx; // im
        self
    }

    pub fn magsq(&self) -> f64 {
        self.real*self.real + self.imag*self.imag
    }

    pub fn cullable(&self) -> bool {
        // in first bulb || in main cardoid
        let c = self.subi(0.25,0.);
        let xysq = c.magsq().sqrt();
        self.subi(1.,0.).magsq() < 0.125 || // smaller cardioid
            0.5*(1.-c.real/xysq) >= xysq  //main cardioid
        // TODO move calcs to after first check
    }

    pub fn map_to_pixel(&self) -> (u32, u32) {
        (((self.real + 1.5) / 2. * WIDTH) as u32,
         ((self.imag + 1.0) / 2. * HEIGHT) as u32)
    }

    pub fn from_pixel(x: usize, y: usize) -> Complex {
        Complex {
         real: x as f64 /WIDTH  * 2. - 1.5,
         imag: y as f64 /HEIGHT * 2. - 1.0,
        }
    }
}

pub fn pixel_to_plane(x: usize, y: usize) -> (f64, f64) {
    (x as f64 / WIDTH * 2. - 1.5, y as f64 / HEIGHT * 2. - 1.)
}

pub fn mutate(rng: &mut ThreadRng, c: Complex, iter_frac: f64) -> Complex {
    if rng.gen_range(0.0,1.0) > iter_frac {
        gen_sample(rng)
    } else {
        let mut_del: f64 = iter_frac * 0.05;
        c.addi(rng.gen_range(-mut_del,mut_del),rng.gen_range(-mut_del,mut_del))
    }
}

pub fn mutate_from_list(rng: &mut ThreadRng, c: Complex, iter_frac: f64, sample_list: &Vec<(f64,f64,f64,f64)>) -> Complex {
    if rng.gen_range(0.0,1.0) > iter_frac {
        gen_sample_from_list(rng,&sample_list)
    } else {
        let mut_del: f64 = iter_frac * 0.05;
        c.addi(rng.gen_range(-mut_del,mut_del),rng.gen_range(-mut_del,mut_del))
    }
}

pub fn gen_sample(rng: &mut ThreadRng) -> Complex {
    let mut s = Complex::new(rng.gen_range(LX,UX),rng.gen_range(LY,UY));
    while s.cullable() {
        s = Complex::new(rng.gen_range(LX,UX),rng.gen_range(LY,UY));
    }
    s
}

pub fn gen_sample_from_list(rng: &mut ThreadRng, sample_list: &Vec<(f64, f64, f64, f64)>) -> Complex {
    let (lx,ux,ly,uy) = sample_list.choose(rng).unwrap();
    let mut s = Complex::new(rng.gen_range(lx,ux),rng.gen_range(ly,uy));
    let mut tries = 5;
    while s.cullable() && tries > 0 {
        tries -= 1;
        s = Complex::new(rng.gen_range(lx,ux),rng.gen_range(ly,uy));
    }
    s
}

/* Mapping Functions */
pub fn hits_to_col_sqrt(val: u32, max: u32) -> u8 { //3rd root gives better results
    ((val as f64 / max as f64).powf(1./3.)  * 255.) as u8
}

pub fn hits_to_col_lin(val: u32, max: u32) -> u8 {
    ((val as f64 / max as f64) * 255.) as u8
}

/* Consts */
// Image bounds
const WIDTH: f64 = 1000.;
const HEIGHT: f64 = 1000.;

// XY plane bounds
const UX: f64 = 0.75;
const LX: f64 = -3.0;

const UY: f64 = 1.0;
const LY: f64 = -1.0;

// Sampling grid bounds
const GRID_RES: usize = 100;
const GRID_CELL_WIDTH: usize = (WIDTH / GRID_RES as f64) as usize;
const GRID_CELL_HEIGHT: usize = (HEIGHT / GRID_RES as f64) as usize;

fn main() {
    // Base mandelrot extends from x = -2 to 0.5
    //                             y = -1 to 1
    let sample_count: u64 = 20_000_000;
    let max_iters = 1000;
    let min_iters = max_iters/3;
    let mut max_hits = 0;
    let mut iters = 0;

    let mut img_buff: ImageBuffer<Rgb<u8>,Vec<u8>> = ImageBuffer::new(WIDTH as u32,HEIGHT as u32);
    let mut hit_buff = Array::<u32,_>::zeros((WIDTH as usize,HEIGHT as usize).f()); // kinda a weird way to init an array but its in the demo
    let mut sample_grid = Array::<u32,_>::zeros((GRID_RES as usize,GRID_RES as usize).f());

    let mut hit_seq: Vec<(u32,u32)> = Vec::new();
    hit_seq.reserve(max_iters);

    println!("Creating sample grid: {} x {} ",GRID_RES,GRID_RES);
    /* Determing good points to sample */
    for ((x,y),point) in sample_grid.indexed_iter_mut() {
        //println!("x: {:?} y: {:?}",x * GRID_CELL_WIDTH ,y * GRID_CELL_HEIGHT);
        let mut iters = 0;
        let mut z = Complex::new(0.,0.);
        let c = Complex::from_pixel(x * GRID_CELL_WIDTH,y * GRID_CELL_HEIGHT);
        while iters < max_iters && z.magsq() < 4. {
            z.square().add(&c);
            iters += 1;
        }

        // mark appropriate sample points
        if iters < max_iters && iters >= min_iters {
            *point = 1; // TODO make a more complicated metric (maybe weight?)
        }
    }

    println!("Generating sampling list...");

    let mut sample_list: Vec<(f64, f64, f64 ,f64)> = Vec::new();
    for (i,sample_box) in sample_grid.windows((2,2)).into_iter().enumerate() {
        let col = i % (GRID_RES - 1) * GRID_CELL_WIDTH;
        let row = i / (GRID_RES - 1) * GRID_CELL_HEIGHT;

        if sample_box.sum() > 0 {
            let (lx,ly) = pixel_to_plane(row,col);
            let (ux,uy) = pixel_to_plane(row+1,col+1);
            sample_list.push((lx,ux,ly,uy));
        }
    }
    println!("Generated sample list.");


    let mut rng = thread_rng();
    let mut c: Complex = gen_sample_from_list(&mut rng,&sample_list);
    println!("Starting sampling");
    let sample_start = Instant::now();

    print!("0%\tdone");
    for s in 0..sample_count{
        let mut z = Complex::new(0.,0.);
        iters = 0;

        while  z.magsq() < 4. && iters < max_iters  {
            z.square().add(&c);
            let (x,y) = z.map_to_pixel();
            iters+=1;

            if x >= WIDTH as u32|| y >= HEIGHT as u32 {
                continue; // interesting choice here... stay or leave?
            }
            hit_seq.push(z.map_to_pixel()); // this will be bad for lots of iterations
        }

        if iters < max_iters && iters >= min_iters && hit_seq.len() > 0 {
            for (x,y) in hit_seq.drain(1..) {
                let hits = hit_buff[[x as usize,y as usize]] + 1;
                if hits > max_hits {
                    max_hits = hits;
                }
                hit_buff[[x as usize, y as usize]] = hits;
            }
        } else {
            hit_seq.clear();
        }

        if s % 100 == 0 {
            print!("\r{:3.1}%\tdone",(s as f64 / sample_count as f64)*100.);
        }

        c = gen_sample(&mut rng);
    }

    println!("\nFinished sampling with {} seconds.\nMapping values.",sample_start.elapsed().as_secs_f32());

    for (x, y, pixel) in img_buff.enumerate_pixels_mut() {
        let mapped_val: u8 =
            hits_to_col_sqrt(hit_buff[[x as usize, y as usize]],max_hits);

        //*pixel = Rgb([mapped_value;3]);
        *pixel = Rgb([mapped_val;3]);
    }

    println!("Saving image with {} seconds.",sample_start.elapsed().as_secs_f32());
    img_buff.save("output/fractal.png").unwrap();
    println!("{} seconds",sample_start.elapsed().as_secs_f32());
}
