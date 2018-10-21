use std::cmp::Ordering;
use std::sync::{
  Arc,
  Mutex
};
use std::sync::mpsc::{
  channel,
  Sender,
  Receiver,
  TryRecvError
};
use std::time::Instant;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

static MS_PER_FLOOR: u64 = 1000;

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

pub enum Message {
    NewOrder(Order),
    SetDirection(ElevatorDirection),
    GoToFloor(Floor),
    Terminate,
}

#[derive(Copy, Clone, Debug)]
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

struct ElevatorHandle {
  id: usize,
  sender: Sender<Message>,
  thread: thread::JoinHandle<()>
}

impl ElevatorHandle {
  pub fn join(self) {
    self.thread.join().unwrap()
  }
}

pub struct Elevators {
  elevators: Vec<ElevatorHandle>,
  floors: &'static Vec<Floor>
}

impl Elevators {
  pub fn new(size: usize, default_floor: Floor, floors: &'static Vec<Floor>) -> Elevators {
    assert!(size > 0);

    let mut elevators = Vec::with_capacity(size);

    for id in 0..size {
      let (sender, receiver) = channel();
      let receiver = Arc::new(Mutex::new(receiver));
      let thread = Elevator::spawn(id, receiver.clone(), default_floor, floors);
      elevators.push(ElevatorHandle { id, sender, thread });
    }

    Elevators {
      elevators,
      floors
    }
  }

  pub fn submit_order(&self, order: Order) {
    let elevator = self.elevators.get(0).unwrap();
    elevator.sender.send(Message::NewOrder(order.clone())).unwrap();
  }

  pub fn wait(&mut self) {
    let mut elevators = Vec::with_capacity(self.elevators.len());
    elevators.append(&mut self.elevators);

    for elevator in elevators {
      println!("Shutting down elevator {}", elevator.id);
      elevator.join();
    }
  }
}

impl Drop for Elevators {
  fn drop(&mut self) {
    println!("Sending terminate message to all elevators.");

    for elevator in &mut self.elevators {
      elevator.sender.send(Message::Terminate).unwrap();
    }

    println!("Shutting down all elevators.");

    let mut elevators = Vec::with_capacity(self.elevators.len());
    elevators.append(&mut self.elevators);

    for elevator in elevators {
      println!("Shutting down elevator {}", elevator.id);
      elevator.join();
    }
  }
}

#[derive(Debug)]
pub struct Elevator {
  pub id: usize,
  cur_dir_queue: Vec<Order>,
  next_dir_queue: Vec<Order>,
  pub direction: ElevatorDirection,
  pub current_floor: Floor,
  floors: &'static Vec<Floor>
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
  pub fn new(id: usize, current_floor: Floor, floors: &'static Vec<Floor>) -> Elevator {
    Elevator {
      id,
      cur_dir_queue: Vec::with_capacity(floors.len()),
      next_dir_queue: Vec::with_capacity(floors.len()),
      direction: ElevatorDirection::Stopped,
      current_floor,
      floors
    }
  }

  pub fn spawn(id: usize, receiver: Arc<Mutex<Receiver<Message>>>, current_floor: Floor, floors: &'static Vec<Floor>) -> thread::JoinHandle<()> {
    let thread = thread::spawn(move || {
      let mut elevator = Elevator::new(id, current_floor, floors);

      loop {
        match receiver.lock().unwrap().try_recv() {
          Ok(message) => match message {
            Message::GoToFloor(floor) => elevator.go_to_floor(floor),
            Message::NewOrder(order) => elevator.queue_order(order),
            Message::SetDirection(direction) => elevator.set_direction(direction),
            Message::Terminate => {
              println!("Elevator {} was told to terminate.", id);
              break;
            },
          },
          Err(TryRecvError::Empty) => {
            elevator.next();
            sleep(Duration::from_millis(MS_PER_FLOOR));
          },
          Err(_) => break
        };
      }
    });

    thread
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
