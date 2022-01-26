use rand::prelude::*;

use std::time::Duration;
use rand::Rng;
use smallvec::{SmallVec, smallvec};

pub trait CubeLight {
    fn light(&mut self, cube : CubeState);
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Location {
    x : usize,
    y : usize,
    z : usize
}

#[derive(Copy, Clone)]
pub struct Color {
    r : u8,
    g : u8,
    b : u8
}

#[derive(Copy, Clone)]
pub struct PixelState {
    color: Color,
    bright: u8,
}

type Range = [ usize ; 2 ];

type CubeState = SmallVec<[(Location, PixelState); 5]>;

fn plane(x_range : Range, y_range : Range, z_range : Range, pixel_state: PixelState) -> CubeState {

    let mut cube = smallvec![];

    for z in z_range[0]..z_range[1] {
        for y in y_range[0]..y_range[1] {
            for x in x_range[0]..x_range[1] {

                let x1 = (Location { x, z, y : 4 -y }, pixel_state);
                cube.push(x1);
            }
        }
    }
    cube
}

fn to_range(n : usize ) -> [usize; 2] {
    [n,n+1]
}

fn gen_plane_y(y : usize, state : PixelState) -> CubeState {
    plane(
        [0,5],
        to_range(y),
        [0,5],
        state
    )
}

fn gen_plane_x(x : usize, state : PixelState) -> CubeState {
    plane(
        to_range(x),
        [0,5],
        [0,5],
        state
    )
}

fn gen_plane_z(z : usize, state : PixelState) -> CubeState {
    plane(
        [0,5],
        [0,5],
        to_range(z),
        state
    )
}

fn empty_pixel() -> PixelState {
    PixelState {
        color : Color { r: 0, g: 0, b: 0},
        bright : 0
    }
}

fn empty_cube() -> CubeState {

    plane([0,5],[0,5],[0,5],empty_pixel())
}

pub struct ConsoleDriver {
    pub state : CubeState
}

fn find_location_index(cube: &CubeState, location: Location) -> usize {

    let mut i = 0;
    for pixel in cube {

        if pixel.0 == location {
            return i;
        }
        i += 1;
    }

    panic!("Location not in Cube")
}

fn replace(cube: &CubeState, new_pixel: (Location, PixelState)) -> CubeState {

    let index = find_location_index(cube, new_pixel.0);

    let mut new_cube : CubeState = smallvec![];

    let mut i = 0;

    for pixel in cube {

        if i == index {
            new_cube.push(new_pixel.clone())
        } else {
            new_cube.push(pixel.to_owned())
        }

        i+=1;
    }
    new_cube
}

fn simple_cube_state_print(cube : CubeState) {

    println!("z,y,x -> Brightness");
    for pixel in cube {
        println!("{},{},{} -> {}", pixel.0.z, pixel.0.y, pixel.0.x, &pixel.1.bright);
    }
}

fn generic_cube_transformer<F>(transformer_fn: F, init_cube : &CubeState, loc: Location, state: PixelState) -> CubeState
    where F: Fn(&CubeState, Location, PixelState) -> CubeState {

    transformer_fn(init_cube, loc, state)
}

fn location_transformer<F>(location_fn : F, loc : Location) -> Option<Location>
    where F: Fn(Location) -> Option<Location> {

    location_fn(loc)
}

fn location_cube_transformer<F>(transformer_fn: F, init_cube : &CubeState, loc: Location, state: PixelState, location_fn: fn(l : Location) -> Option<Location>) -> CubeState
    where F: Fn(&CubeState, Location, PixelState, fn(l : Location) -> Option<Location>) -> CubeState {

    transformer_fn(init_cube, loc, state, location_fn)
}

fn selector<F>(selector_fn: F, cube : &CubeState, loc: Location, state: PixelState) -> Option<Location>
    where F: Fn(&CubeState, Location, PixelState) -> Option<Location> {

    selector_fn(cube, loc, state)
}

fn inc_y_location_transform_to_empty(old_location : Location) -> Option<Location> {
    let mut new_location =  old_location.clone();
    let new_y = old_location.y + 1;

    return if new_y <= 4 {
        new_location.y = new_y;
        Option::Some(new_location)
    } else {
        Option::None
    }
}

fn inc_y_location_transform_write_over(old_location : Location) -> Option<Location> {
    let mut new_location =  old_location.clone();
    let new_y = old_location.y + 1;

    if new_y <= 4 {
        new_location.y = new_y;
    } else {
        new_location.y = new_y % 4;
    }
    return Option::Some(new_location);
}

fn plus_minus_1 (n : usize) -> usize {

    match rng.gen() {
        0 => core + 1,
        1 => n - 1
    }
}

fn random_walker_transform_write_over(old_location : Location) -> Option<Location> {
    let mut new_location =  old_location.clone();

    let mut rng = rand::thread_rng();
    let rand: i64 = rng.gen_range(0..2);

    let new_loc : Location = match rand {
        0 => { // For Z
            Location { x : old_location.x, y : old_location.y , z : plus_minus_1(old_location.z)   }
        }
        1 => { // For x
            Location { x : plus_minus_1(old_location.x), y : old_location.y , z : old_location.z   }
        }
        2 => { // For y
            Location { x : old_location.x, y : plus_minus_1(old_location.y) , z : old_location.z   }
        }
        _ => {
            old_location
        }
    };

    return Option::Some(new_location);
}

fn transformer_replace_location_generator(location_fn: fn(l : Location) -> Option<Location>) -> Box<dyn Fn(&CubeState, Location, PixelState) -> CubeState> {

    return Box::new(move |init_cube : &CubeState, location: Location, pixel : PixelState | {

        let mut new_cube : CubeState = smallvec![];

        let replacement_pixel = (location.clone(), empty_pixel());
        new_cube.push(replacement_pixel);

        match location_fn(location) {
            Some(new_location) => { new_cube.push((new_location, pixel.clone())) }
            None => {}
        };

        new_cube
    });
}


fn test() {

    let bright_selector = |cube : &CubeState, l : Location, pixel : PixelState | if pixel.bright > 0 {  Some(l)  } else { None };

    let inc_y_transformer_gen = transformer_replace_location_generator(inc_y_location_transform_to_empty);

    let inc_y_transformer_write_over_gen = transformer_replace_location_generator(inc_y_location_transform_write_over);


    let pong_transformer = |init_cube : &CubeState, location: Location, pixel : PixelState | {

        let mut new_cube : CubeState = smallvec![];

        let replacement_pixel = (location.clone(), empty_pixel());
        new_cube.push(replacement_pixel);

        match location_fn(location) {
            Some(new_location) => { new_cube.push((new_location, pixel.clone())) }
            None => {}
        };

        new_cube
    };

    let mut test_cube : CubeState = smallvec![];

    let mut pixel = empty_pixel();
    pixel.bright = 255;

    test_cube.push((Location { x : 2, y : 2, z : 2 }, pixel));

    test_cube.push((Location { x : 0, y : 0, z : 0 } , pixel));

    test_cube.push((Location { x : 4, y : 3, z : 4 } , pixel));

    let mut driver = ConsoleDriver::init();

    driver.light(test_cube);
    println!("--------");
    simple_cube_state_print(driver.select(bright_selector));
    println!("--------");

    //driver.updater(bright_selector, inc_y_transformer);

    driver.updater(bright_selector, inc_y_transformer_gen);



    let mover = transformer_replace_location_generator(random_walker_transform_write_over);

    let mut driver2 = ConsoleDriver::init();

    let mut test_cube2 : CubeState = smallvec![];
    test_cube2.push((Location { x : 2, y : 2, z : 2 }, pixel));
    driver.light(test_cube2);

    loop {
        std::thread::sleep(Duration::from_millis(100) );
        //driver.updater(bright_selector, mover);
    }
    simple_cube_state_print(driver.select(bright_selector));
}

impl CubeLight for ConsoleDriver {

    fn light(&mut self, cube : CubeState) {

        if self.state.len() == 0 {
            self.state = empty_cube()
        }

        for pixel in cube {
            self.state = replace(&mut self.state, pixel)
        }
        println!("------------------");
        self.print_cube_state();
    }
}

impl ConsoleDriver {
    fn print_cube_state(&mut self) {

        let mut i = 1;
        for pixel in &self.state {

            print!("{}", if convert_state_to_simple(&pixel.1) { "X" } else { "0" });

            if i % 5 == 0 {
                print!(" -> z:{}, y:{} \n", pixel.0.z, pixel.0.y);
            }

            if i % 25 == 0 {
                println!()
            }
            i +=1;
        }
    }

    fn init() -> ConsoleDriver {

        return ConsoleDriver {
            state: smallvec![]
        };
    }

    fn select(&self, selector: fn(&CubeState, Location, PixelState) -> Option<Location>) -> CubeState {

        apply_selector(&self.state, selector)
    }

    /*fn transformer(&self, transformer : fn(&CubeState, Location, PixelState) -> CubeState) -> CubeState {

        apply_transformer(&self.state, transformer)
    }*/

    fn updater(&mut self,
               selector: fn(&CubeState, Location, PixelState) -> Option<Location>,
               transformer: Box<dyn Fn(&CubeState, Location, PixelState) -> CubeState>) {

        let new_cube = apply_transformer(&self.select(selector), transformer);

        self.light(new_cube)
    }
}

fn apply_transformer(cube : &CubeState, transformer : Box<dyn Fn(&CubeState, Location, PixelState) -> CubeState>) -> CubeState {

    let mut new_cube : CubeState = smallvec![];
    for pixel in cube {
        for pixel in transformer(&cube, pixel.0, pixel.1) {
            new_cube.push(pixel);
        }
    }
    new_cube
}

fn apply_selector(cube : &CubeState, selector: fn(&CubeState, Location, PixelState) -> Option<Location>) -> CubeState {

    let mut filtered_cube_state : CubeState = smallvec![];

    for pixel in cube {

        let xb = selector(&cube, pixel.0, pixel.1);
        match xb {
            None => {}
            Some(l) => { filtered_cube_state.push(pixel.to_owned()) }
        }
    }
    filtered_cube_state
}

type SimplePixelState = bool;

fn convert_state_to_simple(pixel : &PixelState) -> SimplePixelState {
    pixel.bright > 0
}

#[cfg(test)]
mod tests {
    use crate::test;

    #[test]
    fn it_works() {
        test();
    }
    // Idea rain drops
    //
}
