use rand::Rng;
use console::Term;

fn main() {
    let mut game = Game::new(4);
    game.main_loop();
}

enum Move {
    Up,
    Down,
    Left,
    Right,
}

struct Game {
    board: Vec< u32 >,
    size: usize,
    printwidth: usize
}

impl Game {

    // size is length of one side of square board
    fn new(size: usize) -> Game {
        Game {
            board : vec![0; size*size],
            size : size,
            printwidth : 1,
        }
    }

    // Sets game board to all 0's
    fn reset(&mut self) {
        // Create iterator that can mutate elements of vector
        self.board.iter_mut()
            // Set each element to 0
            .for_each(|x| *x=0 as u32);
    }

    // Return a vector of indexes of empty squares
    fn empty_squares(&self) -> Vec< usize > {
        let mut empties = Vec::new();
        for (i, item) in self.board.iter().enumerate() {
            if *item == 0 as u32 {
                empties.push(i);
            }
        }
        empties
    }

    // Set a random empty square to either 2 or 4. Returns index of the set square
    fn set_rand_empty(&mut self) -> Option< usize > {
        // Get a list of indexes with empty squares
        let empties = self.empty_squares();
        // If there are no empty squares, return None
        if empties.len() == 0 { return None }
        // Pick a random empty square to fill
        let idx = rand::thread_rng().gen_range(0..empties.len());
        // Pick a random value 2 or 4
        let val = rand::thread_rng().gen_range(1..=2) * 2;
        // Set empty square and return
        self.board[empties[idx]] = val;
        Some(empties[idx])
    }

    // Process a move. Returns the number of spaces that fell
    fn process_move(&mut self, dir: Move) -> u32 {
        let take = self.size;
        // Set the parameters for creating an iterator that steps through
        // the board like a 2d array
        let (item_step, row_step, reverse) = match dir {
            Move::Left  => (1, self.size, false),
            Move::Right => (1, self.size, true),
            Move::Up  => (self.size, 1, false),
            Move::Down    => (self.size, 1, true),
        };
        // Counter for number of fallen squares
        let mut fallen = 0;
        // Here a row refers to elements that may fall on each other for the
        // given direction.
        'row_for: for skip in (0..row_step*take).step_by(row_step) {
            { // This block forces stepper to go out of scope before next use
                // Boxing stepper lets us be ambiguous about whether the
                // iterator is iterating forward or in reverse, though it does
                // utilize the heap
                // if-else assignment
                let mut stepper: Box<dyn Iterator<Item=&mut u32>> =
                    if reverse {
                        Box::new(self.board
                                     .iter_mut()
                                     .skip(skip)
                                     .step_by(item_step)
                                     .take(take)
                                     .rev())
                    } else {
                        Box::new(self.board
                                     .iter_mut()
                                     .skip(skip)
                                     .step_by(item_step)
                                     .take(take))
                    };
                // Find first nonzero item in row
                let mut collidee = 'nonzero: loop {
                    match stepper.next(){
                        // There are no nonzero items, nothing to do
                        None => continue 'row_for,
                        Some(x) => if *x != 0 { break 'nonzero x }
                    };
                };
                'collisions: loop {
                    // Find next nonzero item in row
                    let collider = 'nonzero: loop {
                        match stepper.next(){
                            None => break 'collisions,
                            Some(x) => if *x != 0 { break 'nonzero x }
                        };
                    };
                    // If they are the same, make second item fall and
                    // collide into first
                    if *collidee == *collider {
                        *collidee += *collider;
                        *collider = 0;
                        // Update printwidth in case new value is bigger than
                        // one we've printed before. printwidth should only
                        // grow
                        let collider_printwidth = (collidee.ilog10() + 1)as usize;
                        self.printwidth = if collider_printwidth > self.printwidth
                            { collider_printwidth } else {self.printwidth};
                        // If there is a collion, the collider fell
                        fallen += 1;
                    }
                    collidee = collider;
                }
            }
            // Reset stepper to process falling squares
            let mut stepper: Box<dyn Iterator<Item=&mut u32 >>;
            if reverse {
                stepper = Box::new(self.board.iter_mut().skip(skip).step_by(item_step).take(take).rev());
            } else {
                stepper = Box::new(self.board.iter_mut().skip(skip).step_by(item_step).take(take));
            }
            // Keep a list of zero spaces to fall to
            let mut fall_spaces = Vec::new();
            let mut fall_space_idx = 0;
            // Find first zero space
            'zero: loop {
                match stepper.next(){
                    // If no zero spaces then nothing to fall to
                    None => continue 'row_for,
                    Some(x) => if *x == 0 {
                        // Add zero space to list
                        fall_spaces.push(x);
                        break 'zero
                    }
                };
            };
            'fall: loop {
                // Find next nonzero space
                let to_fall = 'nonzero: loop {
                    match stepper.next(){
                        None => break 'fall,
                        Some(x) => if *x != 0 {
                            break 'nonzero x
                        } else {
                            // Add any found zero spaces
                            fall_spaces.push(x)
                        }
                    };
                };
                // Move it to zero space as though it fell
                *fall_spaces[fall_space_idx] = *to_fall;
                fall_space_idx += 1;
                *to_fall = 0;
                fall_spaces.push(to_fall);
                fallen += 1;
            }
        }
        fallen
    }

    // Main game loop
    fn main_loop(&mut self) {
        let stdout = Term::buffered_stdout();
        self.reset();
        self.set_rand_empty();
        self.set_rand_empty();
        println!("{self}");
        'game_loop: loop {

            let mv = if let Ok(character) = stdout.read_char() {
                match character {
                    'l' => Move::Up,
                    'w' => Move::Up,
                    'j' => Move::Left,
                    'a' => Move::Left,
                    'k' => Move::Down,
                    's' => Move::Down,
                    ';' => Move::Right,
                    'd' => Move::Right,
                    _ => break 'game_loop,
                }
            } else {
                Move::Down
            };
            if self.process_move(mv) > 0 {
                self.set_rand_empty();
                println!("{self}");
            // lazy loss condition, doesn't check for if a collision could be made
            } else if self.empty_squares().len() == 0 {
                println!("You lose! Score: {}", self.board.iter().max().unwrap());
                return;
            };
        }
    }

}
impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.size;
        let printwidth = self.printwidth;
        for y in 0..size {
            for x in 0..size {
                let idx = x + y*size;
                if self.board[idx] == 0 {
                    write!(f, "{:>printwidth$} ", '-')?;
                } else {
                    write!(f, "{:>printwidth$} ", self.board[idx])?;
                }
            }
            write!(f, "\n")?;
        }
        write!(f, "\n")
    }
}
