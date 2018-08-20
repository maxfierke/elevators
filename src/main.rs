#[macro_use]
extern crate lazy_static;
extern crate rand;

mod elevator;

use elevator::{
    Elevators,
    Floor,
    Order
};
use rand::Rng;

lazy_static! {
    static ref FLOORS: Vec<Floor> = vec![
        Floor::new(-5),
        Floor::new(-4),
        Floor::new(-3),
        Floor::new(-2),
        Floor::new(-1),
        Floor::new(0),
        Floor::new(1),
        Floor::new(2),
        Floor::new(3),
        Floor::new(4),
        Floor::new(5),
        Floor::new(6),
        Floor::new(7),
        Floor::new(8),
        Floor::new(9),
        Floor::new(10),
        Floor::new(11),
        Floor::new(12)
    ];
}

fn gen_random_order(floors: &Vec<Floor>) -> Order {
    let floor_num = rand::thread_rng().gen_range(0, floors.len());
    let passengers = rand::thread_rng().gen_range(0, 8);
    let floor = floors[floor_num];

    Order::new(floor, passengers)
}

fn main() {
    let start_num = rand::thread_rng().gen_range(0, FLOORS.len());
    let start_floor = FLOORS[start_num];
    let mut elevators = Elevators::new(1, start_floor, &FLOORS);

    for _ in 0..48 {
      let order = gen_random_order(&FLOORS);
      elevators.submit_order(order);
    }

    elevators.wait()
}
