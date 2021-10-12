use std::collections::{HashMap, HashSet};
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
}

struct Frame {
    c: HashSet<String>,
    v: HashSet<String>,
    d: HashSet<String>,
    f: Vec<(String, String)>,
    f_labels: HashMap<String, String>,
    e: Vec<String>,
    e_labels: HashMap<String, String>,
}

struct FrameStack {
    list: Vec<Frame>,

}

impl FrameStack {
    fn push(&mut self, token: String) {
        self.list.push(token)
    }

    fn add_c(&mut self, token: String) {
        let frame = self.list.last().unwrap();

        if frame.c.contains(&token) {
            panic!("Const already defined")
        }
        if frame.v.contains(&token) {
            panic!("consta elaryd defined as var in scope")
        }
        frame.c.insert(token);
    }

    fn add_v(&mut self, token: String) {
        let frame = self.list.last().unwrap();

        if frame.c.contains(&token) {
            panic!("Variable already defined")
        }
        if frame.v.contains(&token) {
            panic!("Variable elaryd defined as var in scope")
        }
        frame.v.insert(token);
    }

    fn add_f(&mut self, var: String, kind: String, label: String) {
        if !self.lookup_v(var) {
            panic!("var not defined")
        }
        if !self.lookup_c(kind) {
            panic!("const not defined")
        }

        let frame = self.list.last().unwrap();
        if frame.contains_key(var) {
            panic!("f already defined in scope")
        }
        frame.f.push((var, kind));
        frame.f_labels[var] = label;
    }

    fn add_e(&mut self, stat: Vec<String>, label: String) {
        let frame = self.list.last().unwrap();

        frame.e.push(stat);
        frame.e_labels[stat.join(" ")] = label;

    }

    fn add_d(&mut self, stat: Vec<String>) {
        let frame = self.list.last().unwrap();



    }
}

fn main() {
    println!("Hello, world!");
}
