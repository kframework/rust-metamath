use std::{collections::{HashMap, HashSet, VecDeque}, ops::Index, os::unix::process};
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

             if let Ok(num) = result {
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
        stat
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
    dvs: HashSet<(String, String)>,
    f_hyps: VecDeque<(String, String)>,
    e_hyps: Vec<Statement>,
    stat: Statement,
}

impl FrameStack {
    fn push(&mut self) {
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
        frame.f_labels.insert(var, label);
    }

    fn add_e(&mut self, stat: Vec<String>, label: String) {
        let frame = self.list.last_mut().unwrap();

        frame.e.push(stat.clone());
        frame.e_labels.insert(stat, label);

    }

    fn add_d(&mut self, stat: Statement) {
        let frame = self.list.last_mut().unwrap();
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

    fn make_assertion(&mut self, stat: Statement) -> Assertion {
        let _frame = self.list.last_mut().unwrap();

        let e_hyps: Vec<Statement> = self.list.iter().flat_map(|fr| fr.e.clone()).collect();

        let stat_vec = vec!(stat.clone());

        let chained = e_hyps.iter().chain(stat_vec.iter());


        let mut mand_vars : HashSet<&String> = chained.flatten().filter(|tok| self.lookup_v(tok)).collect();


        // this is absolutely terrible.
        // Definetely needs to be redone
        let cartesian : HashSet<(String, String)> = mand_vars.clone().
            into_iter().flat_map(|x| mand_vars.clone().into_iter().map(move |y| (x.clone(), y.clone()))).collect();


        let dvs : HashSet<(String, String)> = self.list.iter().
            flat_map(|fr| fr.d.intersection(&cartesian)).cloned().collect();


        let mut f_hyps = VecDeque::new();
        self.list.iter().rev().for_each(|fr| {
            fr.f.iter().for_each(|(k, v)| {
                if mand_vars.contains(&v) {
                    mand_vars.remove(&v);
                    f_hyps.push_front((k.clone(), v.clone()));
                }
            });
        });

                                   Assertion {
                                         dvs,
                                       f_hyps,
                                       e_hyps,
                                       stat,
                                   }

    }


}

// first one is label type,
type LabelEntry = (String, LabelData);

// the original seems to abuse python's type system to create this,
// ideally I'd use a real AST
enum LabelData {
    Ap(Assertion),
    Ef(Statement),
}
struct MM {
    fs: FrameStack,
    labels: HashMap<String, LabelEntry>,
    begin_label: Option<String>,
    stop_label: String,
}
use crate::LabelData::Ef;
use crate::LabelData::Ap;

impl MM {
    fn new(begin_label: String, stop_label: String) -> MM {
        MM {
            fs: FrameStack::default(),
            labels: HashMap::new(),
            begin_label: Some(begin_label),
            stop_label,
        }
    }

    fn read(&mut self, toks: &mut Tokens) {
        self.fs.push();
        let mut label: Option<String> = None;
        let mut tok = toks.read_comment();
        loop {
            match tok.as_deref() {
                Some("$}") => break,
                Some("$c") => {
                    for tok in toks.readstat() {
                        self.fs.add_c(tok);
                    }
                }
                Some("$v") => {
                    for tok in toks.readstat() {
                        self.fs.add_v(tok);
                    }
                }
                Some("$f") => {
                    let stat = toks.readstat();
                    let label1 = label.clone(); //I'll figure it out later I promise
                    if label1.is_none() {
                        panic!("$f must have label");
                    }
                    if stat.len() != 2 {
                        panic!("$f must have length 2");
                    }
                    let label_u = &label1.unwrap(); //wow I'm bad

                    println!("{} $f {} {} $.", label_u, stat[0].clone(), stat[1].clone());
                    self.fs.add_f(stat[1].clone(), stat[0].clone(), label_u.into());
                    let data = Ef(vec![stat[0].clone(), stat[1].clone()]);
                    self.labels.insert(label_u.to_string(), ("$f".to_string(), data));
                    label = None;
                }
                Some("$a") => {
                    let label1 = label.clone(); //I'll figure it out later I promise
                    if label.is_none() {
                        panic!("$a must hae label")
                    }
                    let label_u = &label1.unwrap();
                    if label_u == &self.stop_label {
                        panic!("exit"); // I don't understad why you would want to exit
                        //
                    }
                    let data = Ap(self.fs.make_assertion(toks.readstat()));
                    self.labels.insert(label_u.to_string(), ("$a".to_string(), data));
                }

                Some("$e") => {
                    let label = label.clone().expect("e must have label");

                    let stat = toks.readstat();
                    self.fs.add_e(stat.clone(), label.clone());
                    let data = Ef(stat);
                    self.labels.insert(label, ("$p".to_string(), data));
                }
                Some("$p") => {
                    let label_u = label.clone().expect("$p must have elabel");
                    if label_u == self.stop_label {
                        std::process::exit(0);
                    }
                    let stat = toks.readstat();
                    let i = stat.iter().position(|x| x == "$=").expect("Mmust have $=");
                    let proof = &stat[i + 1..].to_vec();
                    let stat = &stat[..i];


                    if self.begin_label.is_some() && &label_u == self.begin_label.as_ref().unwrap() {
                        self.begin_label = None;
                    }
                    if self.begin_label.is_none() {
                        println!("verifying {}", label_u);
                        self.verify(label_u.clone(), stat.to_vec(), proof.to_vec());
                    }
                    let data = Ap(self.fs.make_assertion(stat.to_vec()));
                    self.labels.insert(label_u, ("$p".to_string(), data));
                    label = None;
                }
                Some("$d") => {
                    self.fs.add_d(toks.readstat());
                }
                Some("${") => {
                    self.read(toks);
                }
                Some(x) => if !x.starts_with("$") {
                    label = tok;
                }
                Some(_) => {
                    print!("tok {:?}", tok);

                }
                None => break,
            }
            tok = toks.read_comment();


        }
        self.fs.list.pop();
        

    }

    fn apply_subst(&mut self, _stat: Vec<String>, _subst: HashMap<String, String> ) -> Vec<String> {
        todo!();
    }

    //probably wrong type for proof
    fn decompress_proof(&mut self, _stat: Statement, _proof: Vec<String>) -> Vec<String> {
        todo!();
    }

    fn verify(&mut self, _stat_label: String, _stat: Statement, _proof: Vec<String>) {
        todo!();
    }

    fn dump(&mut self) {
        todo!();
    }
}
fn main() {
    println!("Hello, world!");
}
