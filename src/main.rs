use std::{collections::{HashMap, HashSet}};
use std::fs::File;
use std::io::BufReader;
use std::cmp::min;
use std::cmp::max;
use std::io::BufRead;

struct Tokens {
    lines_buffer: Vec<BufReader<File>>,
    token_buffer: Vec<String>,
    imported_files: HashSet<String>,
}

type Statement = Vec<String>;
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

    fn readstat(&mut self) -> Statement {
        let mut stat = vec!();
        let mut token = self.read_comment().unwrap();

        while token != "$." {
            stat.push(token);
            token = self.read_comment().expect("EOF before $.");
        }
        return stat;


    }
}

#[derive(Default, Debug)]
struct Frame {
    c: HashSet<String>,
    v: HashSet<String>,
    d: HashSet<(String, String)>,
    f: Vec<(String, String)>,
    f_labels: HashMap<String, String>,
    e: Vec<Vec<String>>,
    e_labels: HashMap<Statement, String>,
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

    fn add_d(&mut self, stat: Statement) {
        let mut frame = self.list.last_mut().unwrap();
        //let mut product_vec = vec!();
        for x in &stat {
            for y in &stat {
                if x != y {
                    frame.d.insert((min(x.clone(), y.clone()), max(x.clone(), y.clone())));
                }
            }
        }

    }

    fn lookup_c(&mut self, token: &str) -> bool {
        self.list.iter().rev().any(|fr| fr.c.contains(token))
    }

    fn lookup_v(&mut self, token: &str) -> bool {
        self.list.iter().rev().any(|fr| fr.v.contains(token))
    }


    fn lookup_f(&mut self, var: String) -> String {
        let f = self.list.iter().rev().find(|frame| frame.f_labels.contains_key(&var)).unwrap();

        f.f_labels[&var].clone()
    }

    fn lookup_d(&mut self, x: String, y: String ) -> bool {
        self.list.iter().rev().any(|fr| fr.d.contains(&(min(x.clone(), y.clone()), max(x.clone(), y.clone()))))
    }

    fn lookup_e(&mut self, stmt: Statement) -> String {
        let f = self.list.iter().rev().find(|frame| frame.e_labels.contains_key(&stmt)).expect("Bad e");


        f.e_labels[&stmt].clone()
    }

    fn make_assertion(&mut self, stat: String) -> Assertion {
        unimplemented!();
    }

}

struct MM {
    fs: FrameStack,
    labels: HashMap<String, (String, Vec<(String, String)>, Vec<String>)>,
    begin_label: String,
    stop_label: String,
}

impl MM {
    fn new(begin_label: String, stop_label: String) -> MM {
        MM {
            fs: FrameStack::default(),
            labels: HashMap::new(),
            begin_label: begin_label,
            stop_label: stop_label,
        }
    }

    fn read(&mut self, toks: Tokens) {
     todo!();
    }

    fn apply_subst(&mut self, stat: Vec<String>, subst: HashMap<String, String> ) -> Vec<String> {
        todo!();
    }

    //probably wrong type for proof
    fn decompress_proof(&mut self, stat: Statement, proof: Vec<String>) -> Vec<String> {
        todo!();
    }

    fn verify(&mut self, stat_label: String, stat: Statement, proof: Vec<String>) {
        todo!();
    }

    fn dump(&mut self) {
        todo!();
    }
}
fn main() {
    println!("Hello, world!");
}
