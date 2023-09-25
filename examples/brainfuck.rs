use aott::prelude::*;

#[derive(Clone, Debug)]
enum Instruction {
        Left,
        Right,
        Increment,
        Decrement,
        Read,
        Write,
        Loop(Vec<Self>),
}

#[parser]
fn bf_file(input: &str) -> Vec<Instruction> {
        choice((
                // Basic instructions are just single characters!
                just('<').to(Instruction::Left),
                just('>').to(Instruction::Right),
                just('+').to(Instruction::Increment),
                just('-').to(Instruction::Decrement),
                just(',').to(Instruction::Read),
                just('.').to(Instruction::Write),
                // recursion is easy: just put in the function as is!
                delimited(just('['), bf_file, just(']')).map(Instruction::Loop),
        ))
        // Brainfuck is sequential, so we parse as many instructions as is possible
        .repeated()
        .parse_with(input)
}

const TAPE_LENGTH: usize = 10_000;

fn eval(ast: &[Instruction], ptr: &mut usize, tape: &mut [u8; TAPE_LENGTH]) {
        use std::io::Read;
        use Instruction as Insn;

        for sym in ast {
                match sym {
                        Insn::Left => *ptr = (*ptr + TAPE_LENGTH - 1).rem_euclid(TAPE_LENGTH),
                        Insn::Right => *ptr = (*ptr + 1).rem_euclid(TAPE_LENGTH),
                        Insn::Increment => tape[*ptr] = tape[*ptr].wrapping_add(1),
                        Insn::Decrement => tape[*ptr] = tape[*ptr].wrapping_sub(1),
                        Insn::Read => {
                                tape[*ptr] = std::io::stdin().bytes().next().unwrap().unwrap()
                        }
                        Insn::Write => print!("{}", tape[*ptr] as char),
                        Insn::Loop(next_ast) => {
                                while tape[*ptr] != 0 {
                                        eval(next_ast, ptr, tape)
                                }
                        }
                }
        }
}

fn main() {
        // Example brainfuck instructions
        let input_ = "--[>--->->->++>-<<<<<-------]>--.>---------.>--..+++.>----.>+++++++++.<<.+++.------.<-.>>+.";

        // Uncomment to get user input
        // use std::io::BufRead;
        // let input_ = std::io::stdin().lock().lines().next().unwrap().unwrap();

        // Uncomment to read from file
        // let input_ =
        //         std::fs::read_to_string(std::env::args().nth(1).expect("Expected file argument"))
        //                 .expect("Failed to read file");

        #[allow(clippy::useless_asref)]
        let input = input_.as_ref();

        println!("Brainfuck input: {input}");
        let result = bf_file.parse(input);
        match result {
                Ok(ok) => {
                        println!("Parsed Brainfuck AST: {ok:#?}");
                        eval(&ok, &mut 0, &mut [0; TAPE_LENGTH]);
                }
                Err(err) => println!("Parsing error: {err:?}"),
        }
}
