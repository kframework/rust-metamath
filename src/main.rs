use std::{collections::{HashMap, HashSet}};
use std::fs::File;
use std::io::BufReader;

use std::io::BufRead;

struct Tokens {
    lines_buffer: Vec<BufReader<File>>,
    token_buffer: Vec<String>,
    imported_files: HashSet<String>,
}

impl Tokens {
    fn read(&mut self) -> Option<String> {
        while self.token_buffer.is_empty() {
            let mut line = String::new();
            // pretend this succeeeds
            let result = self.lines_buffer.last_mut().unwrap().read_line(&mut line);

            match result {
                Ok(num) => {
                    self.lines_buffer.pop();
                    if num == 0 {
                        self.token_buffer = line.split_whitespace().map(|x| x.into()).collect();
                        self.token_buffer.reverse();
                    } else {
                        self.lines_buffer.pop();
                        if self.lines_buffer.is_empty() {
                            return None;
                        }
                    }
                }
                Err(_) => {}
            }
        }
        self.token_buffer.pop()
    }

    fn read_file(&mut self) -> Option<String> {

        loop {
            let token = self.read();
            let input = token.filter(|x| x == "$[");

            match input {
                Some(_) => {
                    let filename = self.read().unwrap();
                    let endbracket = self.read().unwrap();

                    if endbracket != "$]" {
                        panic!();
                    }

                    if !self.imported_files.contains(&filename) {
                        self.lines_buffer
                            .push(BufReader::new(File::open(filename.clone()).unwrap()));
                        self.imported_files.insert(filename);
                    }
                }
                None => {
                    break;
                }
            };
        }
        self.token_buffer.pop()
    }

    fn read_comment(&mut self) -> Option<String> {
        loop {
            let mut token = self.read_file()?;

            if token == "$(" {
                while token != "$)" {
                    token = self.read()?;
                }
            } else {
                return Some(token);
            }
        }
    }

    fn readstat(&mut self) -> Vec<String> {
        todo!();
    }
}

#[derive(Default, Debug)]
struct Frame {
    c: HashSet<String>,
    v: HashSet<String>,
    d: HashSet<String>,
    f: Vec<(String, String)>,
    f_labels: HashMap<String, String>,
    e: Vec<Vec<String>>,
    e_labels: HashMap<Vec<String>, String>,
}

#[derive(Default, Debug)]
struct FrameStack {
    list: Vec<Frame>,

}

#[derive(Default, Debug)]
struct Assertion {
    dvs: Vec<(String, String)>,
    f_hyps: Vec<String>,
    e_hyps: Vec<String>,
    stat: String,
}

impl FrameStack {
    fn push(&mut self, token: String) {
        self.list.push(Frame::default());
    }

    fn add_c(&mut self, token: String) {
        let frame = &mut self.list.last_mut().unwrap();

        if frame.c.contains(&token) {
            panic!("Const already defined")
        }
        if frame.v.contains(&token) {
            panic!("consta elaryd defined as var in scope")
        }
        frame.c.insert(token);
    }

    fn add_v(&mut self, token: String) {
        let frame = &mut self.list.last_mut().unwrap();

        if frame.c.contains(&token) {
            panic!("Variable already defined")
        }
        if frame.v.contains(&token) {
            panic!("Variable elaryd defined as var in scope")
        }
        frame.v.insert(token);
    }

    fn add_f(&mut self, var: String, kind: String, label: String) {
        if !self.lookup_v(&var) {
            panic!("var not defined")
        }
        if !self.lookup_c(&kind) {
            panic!("const not defined")
        }

        let frame = self.list.last_mut().unwrap();
        if frame.f_labels.contains_key(&var) {
            panic!("f already defined in scope")
        }
        frame.f.push((var.clone(), kind));
        frame.f_labels.insert(var.into(), label);
    }

    fn add_e(&mut self, stat: Vec<String>, label: String) {
        let frame = self.list.last_mut().unwrap();

        frame.e.push(stat.clone());
        frame.e_labels.insert(stat, label);

    }

    fn add_d(&mut self, stat: Vec<String>) {
        let frame = self.list.last().unwrap();
        unimplemented!();
    }

    fn lookup_c(&mut self, token: &str) -> bool {
        unimplemented!();
    }

    fn lookup_v(&mut self, token: &str) -> bool {
        unimplemented!();
    }


    fn lookup_f(&mut self, var: String) -> String {
        unimplemented!();
    }

    fn lookup_d(&mut self, x: String, y: String ) -> bool {
        unimplemented!();
    }

    fn lookup_e(&mut self, var: String) -> String {
        unimplemented!();
    }

    fn make_assertion(&mut self, stat: String) -> Assertion {
        unimplemented!();
    }

}

fn main() {
    println!("Hello, world!");
}
