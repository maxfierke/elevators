use std::cmp::Ordering;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElevatorDirection {
  Up,
  Down,
  Stopped
}

#[derive(Clone, Copy, Debug, Eq)]
pub struct Floor {
  pub num: i8
}

impl Floor {
  pub fn new(num: i8) -> Floor {
    Floor { num }
  }
}

impl Ord for Floor {
    fn cmp(&self, other: &Floor) -> Ordering {
        self.num.cmp(&other.num)
    }
}

impl PartialOrd for Floor {
    fn partial_cmp(&self, other: &Floor) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Floor {
    fn eq(&self, other: &Floor) -> bool {
        self.num == other.num
    }
}

#[derive(Debug)]
pub struct Order {
  floor: Floor,
  num_of_people: u8,
  instant: Instant
}

impl Order {
  pub fn new(floor: Floor, num_of_people: u8) -> Order {
    Order {
      floor,
      num_of_people,
      instant: Instant::now()
    }
  }
}

#[derive(Debug)]
pub struct Elevator {
  cur_dir_queue: Vec<Order>,
  next_dir_queue: Vec<Order>,
  pub direction: ElevatorDirection,
  pub current_floor: Floor
}

fn add_to_queue(queue: &mut Vec<Order>, order: Order) {
  if queue.iter().any(|ref queued| queued.floor == order.floor) {
    return;
  }

  queue.push(order);
}

fn sort_queue(queue: &mut Vec<Order>, direction: ElevatorDirection) {
  if direction == ElevatorDirection::Down {
    queue.sort_by(|a, b| b.floor.cmp(&a.floor));
  } else {
    queue.sort_by(|a, b| a.floor.cmp(&b.floor));
  }
}

impl Elevator {
  pub fn new(floors: &Vec<Floor>, current_floor: Floor) -> Elevator {
    Elevator {
      cur_dir_queue: Vec::with_capacity(floors.len()),
      next_dir_queue: Vec::with_capacity(floors.len()),
      direction: ElevatorDirection::Stopped,
      current_floor
    }
  }

  pub fn debug(&self) {
    println!("{:?}", self);
  }

  pub fn go_to_floor(&mut self, floor: Floor) {
    println!("Going to Floor {}", floor.num);
    self.current_floor = floor;
  }

  pub fn queue_order(&mut self, order: Order) {
    let floor = order.floor;

    if self.direction == ElevatorDirection::Stopped {
      if floor < self.current_floor {
        self.set_direction(ElevatorDirection::Down)
      } else if floor > self.current_floor {
        self.set_direction(ElevatorDirection::Up)
      }
    }

    if (floor < self.current_floor && self.direction == ElevatorDirection::Down) ||
       (floor > self.current_floor && self.direction == ElevatorDirection::Up) {
      println!("Added floor {} to active queue", floor.num);
      add_to_queue(&mut self.cur_dir_queue, order);
      sort_queue(&mut self.cur_dir_queue, self.direction);
    } else if (floor < self.current_floor && self.direction == ElevatorDirection::Up) ||
       (floor > self.current_floor && self.direction == ElevatorDirection::Down) {
      println!("Added floor {} to next queue", floor.num);
      add_to_queue(&mut self.next_dir_queue, order);
    }
  }

  pub fn set_direction(&mut self, direction: ElevatorDirection) {
    println!("Direction changed to {:?} from {:?}", direction, self.direction);
    self.direction = direction;
  }

  fn all_queues_emptied(&self) -> bool {
    self.cur_dir_queue.is_empty() && self.next_dir_queue.is_empty()
  }
}

impl Iterator for Elevator {
  type Item = Floor;

  fn next(&mut self) -> Option<Floor> {
    if self.cur_dir_queue.is_empty() {
      println!("Swapping directional queues");
      self.cur_dir_queue.append(&mut self.next_dir_queue);

      let next_direction = match self.direction {
        ElevatorDirection::Up => ElevatorDirection::Down,
        ElevatorDirection::Down => ElevatorDirection::Up,
        ElevatorDirection::Stopped => {
          if self.cur_dir_queue.is_empty() {
            ElevatorDirection::Stopped
          } else if self.cur_dir_queue[0].floor > self.current_floor {
            ElevatorDirection::Up
          } else if self.cur_dir_queue[0].floor < self.current_floor {
            ElevatorDirection::Down
          } else {
            ElevatorDirection::Stopped
          }
        }
      };

      self.set_direction(next_direction);
      sort_queue(&mut self.cur_dir_queue, self.direction);
    }

    if !self.cur_dir_queue.is_empty() {
      let next_floor = self.cur_dir_queue.remove(0).floor;
      self.go_to_floor(next_floor);
      Some(next_floor)
    } else if self.all_queues_emptied() {
      self.set_direction(ElevatorDirection::Stopped);
      None
    } else {
      self.next()
    }
  }
}
