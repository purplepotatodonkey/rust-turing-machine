use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    // Main cycle
    ScanRight,
    MarkRightDigit,
    FindSpaceGoingLeft,
    AddDigits,
    ReturnRight,
    // Carry handling
    PropagateCarry,
    // Cleanup
    FindStart,
    CleanupMarkers,
    Halt,
}

struct TuringMachine {
    tape: HashMap<i32, char>,
    state: State,
    position: i32,
    blank: char,
    right_digit: u8,
    left_digit: u8,
    carry: u8,
}

impl TuringMachine {
    fn new(input: &str) -> Self {
        let mut tape = HashMap::new();
        for (i, ch) in input.chars().enumerate() {
            tape.insert(i as i32, ch);
        }
        
        TuringMachine {
            tape,
            state: State::ScanRight,
            position: 0,
            blank: '_',
            right_digit: 0,
            left_digit: 0,
            carry: 0,
        }
    }
    
    fn read(&self) -> char {
        *self.tape.get(&self.position).unwrap_or(&self.blank)
    }
    
    fn write(&mut self, c: char) {
        if c == self.blank {
            self.tape.remove(&self.position);
        } else {
            self.tape.insert(self.position, c);
        }
    }
    
    fn step(&mut self) {
        let c = self.read();
        
        match self.state {
            State::ScanRight => {
                if c == '_' {
                    self.position -= 1;
                    self.state = State::MarkRightDigit;
                } else {
                    self.position += 1;
                }
            }
            
            State::MarkRightDigit => {
                match c {
                    'X' => self.position -= 1,  // Skip already processed
                    '0' => {
                        self.right_digit = 0;
                        self.write('X');
                        self.position -= 1;
                        self.state = State::FindSpaceGoingLeft;
                    }
                    '1' => {
                        self.right_digit = 1;
                        self.write('X');
                        self.position -= 1;
                        self.state = State::FindSpaceGoingLeft;
                    }
                    ' ' => {
                        // No more right digits!
                        if self.carry == 1 {
                            self.position -= 1;
                            self.state = State::PropagateCarry;
                        } else {
                            self.state = State::FindStart;
                        }
                    }
                    '_' => self.position -= 1,
                    _ => panic!("Unexpected '{}' in MarkRightDigit", c),
                }
            }
            
            State::FindSpaceGoingLeft => {
                if c == ' ' {
                    self.position -= 1;
                    self.state = State::AddDigits;
                } else {
                    self.position -= 1;
                }
            }
            
            State::AddDigits => {
                self.left_digit = match c {
                    '0' => 0,
                    '1' => 1,
                    'X' | '_' => 0,  // Left number exhausted
                    _ => panic!("Unexpected '{}' in AddDigits", c),
                };
                
                let sum = self.left_digit + self.right_digit + self.carry;
                let result_digit = sum % 2;
                self.carry = sum / 2;
                
                self.write(if result_digit == 0 { '0' } else { '1' });
                self.position += 1;
                self.state = State::ReturnRight;
            }
            
            State::ReturnRight => {
                if c == '_' {
                    self.position -= 1;
                    self.state = State::ScanRight;
                } else {
                    self.position += 1;
                }
            }
            
            State::PropagateCarry => {
                match c {
                    'X' | ' ' | '_' => {
                        self.position -= 1;
                    }
                    '0' => {
                        self.write('1');
                        self.carry = 0;
                        self.state = State::FindStart;
                    }
                    '1' => {
                        self.write('0');
                        self.position -= 1;
                        // carry stays 1
                    }
                    _ => {
                        // Reached beginning, add new digit
                        self.write('1');
                        self.carry = 0;
                        self.state = State::FindStart;
                    }
                }
                
                // Check if we've gone far enough left that we need to add a new digit
                if self.position < -10 && self.carry == 1 {
                    self.write('1');
                    self.carry = 0;
                    self.state = State::FindStart;
                }
            }
            
            State::FindStart => {
                // Move to leftmost non-blank
                if self.position > -20 {
                    self.position -= 1;
                    if self.read() == '_' {
                        self.position += 1;
                        self.state = State::CleanupMarkers;
                    }
                } else {
                    self.position = 0;
                    self.state = State::CleanupMarkers;
                }
            }
            
            State::CleanupMarkers => {
                match c {
                    'X' | ' ' => {
                        self.write('_');
                        self.position += 1;
                    }
                    '_' => {
                        self.state = State::Halt;
                    }
                    _ => {
                        self.position += 1;
                    }
                }
            }
            
            State::Halt => {}
        }
    }
    
    fn run(&mut self, max_steps: usize, verbose: bool) {
        let mut steps = 0;
        
        if verbose {
            println!("Step 0:\n{}\n", self);
        }
        
        while self.state != State::Halt && steps < max_steps {
            self.step();
            steps += 1;
            
            if verbose && steps % 5 == 0 {
                println!("Step {}:\n{}\n", steps, self);
            }
        }
        
        if self.state == State::Halt {
            println!("✓ Halted in {} steps", steps);
        } else {
            println!("⚠ Stopped at {} steps", max_steps);
        }
    }
    
    fn get_result(&self) -> String {
        if self.tape.is_empty() {
            return String::from("0");
        }
        
        let min = *self.tape.keys().min().unwrap();
        let max = *self.tape.keys().max().unwrap();
        
        (min..=max)
            .map(|i| *self.tape.get(&i).unwrap_or(&self.blank))
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }
}

impl fmt::Display for TuringMachine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let min = self.tape.keys().min().copied().unwrap_or(0).min(self.position - 2);
        let max = self.tape.keys().max().copied().unwrap_or(0).max(self.position + 2);
        
        write!(f, "  ")?;
        for i in min..=max {
            write!(f, "{}", *self.tape.get(&i).unwrap_or(&self.blank))?;
        }
        writeln!(f)?;
        
        write!(f, "  ")?;
        for i in min..=max {
            write!(f, "{}", if i == self.position { "^" } else { " " })?;
        }
        writeln!(f)?;
        
        write!(f, "  {:?} | carry={}", self.state, self.carry)
    }
}

fn to_dec(bin: &str) -> usize {
    usize::from_str_radix(bin, 2).unwrap_or(0)
}

fn main() {
    println!("\n╔═══════════════════════════════════════════╗");
    println!("║   TURING MACHINE: Binary Addition        ║");
    println!("╚═══════════════════════════════════════════╝\n");
    
    println!("📖 Key Concepts:");
    println!("  • States: Rules that control behavior");
    println!("  • Tape: Infinite memory (HashMap)");
    println!("  • Head: Current position on tape");
    println!("  • Transitions: (state, symbol) → (new_state, write, move)\n");
    println!("═══════════════════════════════════════════\n");
    
    let tests = vec![
        ("111 11", "1010", 10),
        ("101 11", "1000", 8),
        ("1 1", "10", 2),
        ("1111 1", "10000", 16),
    ];
    
    for (i, (input, expected, expected_dec)) in tests.iter().enumerate() {
        println!("TEST {}: {}", i + 1, input);
        println!("Expected: {} (decimal: {})\n", expected, expected_dec);
        
        let mut tm = TuringMachine::new(input);
        tm.run(500, i == 0);  // Verbose only for first test
        
        let result = tm.get_result();
        let result_dec = to_dec(&result);
        let correct = result == *expected;
        
        println!("Result:   {} (decimal: {}) {}\n", 
                 result, result_dec, if correct { "✓" } else { "✗" });
        println!("═══════════════════════════════════════════\n");
    }
}
