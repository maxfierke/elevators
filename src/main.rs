extern crate rand;

mod elevator;

use elevator::{
    Elevator,
    Floor,
    Order
};
use rand::Rng;

fn gen_random_order(floors: &Vec<Floor>) -> Order {
    let floor_num = rand::thread_rng().gen_range(0, floors.len());
    let passengers = rand::thread_rng().gen_range(0, 8);
    let floor = floors[floor_num];

    Order::new(floor, passengers)
}

fn main() {
    let floors = vec![
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

    let mut elevator = Elevator::new(&floors, floors[1]);

    let start_num = rand::thread_rng().gen_range(0, floors.len());
    elevator.go_to_floor(floors[start_num]);

    for _ in 0..48 {
        let order = gen_random_order(&floors);
        elevator.queue_order(order);
    }

    for floor in elevator {
        println!("Opening on {}", floor.num);
    }
}
