use std::{collections::{HashMap, HashSet, VecDeque}};
use std::fs::File;
use std::io::BufReader;
use std::cmp::min;
use std::cmp::max;
use std::io::BufRead;


#[derive(Debug)]
struct Tokens {
    lines_buffer: Vec<BufReader<File>>,
    token_buffer: Vec<String>,
    imported_files: HashSet<String>,
}

type Statement = Vec<String>;
impl Tokens {
    fn new(lines: BufReader<File>) -> Tokens {
        Tokens {
            lines_buffer :  vec!(lines),
            token_buffer : vec!(),
            imported_files : HashSet::new(),
        }
    }
    fn read(&mut self) -> Option<String> {
        println!("inside read function with state {:?}", self);
        while self.token_buffer.is_empty() {
            println!("Buffer is empty, refilling");
            let mut line = String::new();
            // pretend this succeeeds
            let result = self.lines_buffer.last_mut().unwrap().read_line(&mut line);
            println!("Read line: {}", line);

            match result {
                Ok(num) if num > 0 => {
                    println!("Read {} lines ", num);
                    self.token_buffer = line.split_whitespace().map(|x| x.into()).collect();
                    self.token_buffer.reverse();

                }
                _ => {
                    println!("Done with file");
                    self.lines_buffer.pop();
                    if self.lines_buffer.is_empty() {
                        return None;
                    }

                }
            }
            println!("Created token buffer {:?}", self.token_buffer);
        }
        self.token_buffer.pop()
    }

    fn read_file(&mut self) -> Option<String> {
        println!("reading file");

        let token = self.read();
        println!("In read file found token {:?}", token);
        loop {


            match token.as_deref() {
                Some("$[") => {
                    let filename = self.read().expect("Couldn't find filename");
                    let endbracket = self.read().expect("Coludn't find end bracket");

                    if endbracket != "$]" {
                        panic!("End bracket not found");
                    }

                    if !self.imported_files.contains(&filename) {
                        println!("Found new file {}", &filename);

                        self.lines_buffer
                            .push(BufReader::new(File::open(filename.clone()).expect("Failed to open file")));
                        self.imported_files.insert(filename);
                    }
                }
                _ => {
                    break;
                }
            };
        }
        token
    }

    fn read_comment(&mut self) -> Option<String> {
        println!("reading comment");

        loop {
            let mut token = self.read_file();
            println!("In read comment: found token to be {:?}", token);
            match &token {
                None => return None,
                Some(x) if x == "$(" => {
                    loop {
                        match token.as_deref() {
                            Some("$)") => break,
                            _ => token = self.read(),
                        }
                    }
                }
                _ => return token,
            }
        }
    }

    fn readstat(&mut self) -> Statement {
        let mut stat = vec!();
        let mut token = self.read_comment().unwrap();

        println!("In read stat, found token to be {:?}", token);
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

    fn lookup_c(&self, token: &str) -> bool {
        self.list.iter().rev().any(|fr| fr.c.contains(token))
    }

    fn lookup_v(&self, token: &str) -> bool {
        self.list.iter().rev().any(|fr| fr.v.contains(token))
    }


    fn lookup_f(&self, var: String) -> String {
        let f = self.list.iter().rev().find(|frame| frame.f_labels.contains_key(&var)).unwrap();

        f.f_labels[&var].clone()
    }

    fn lookup_d(&mut self, x: String, y: String ) -> bool {
        self.list.iter().rev().any(|fr| fr.d.contains(&(min(x.clone(), y.clone()), max(x.clone(), y.clone()))))
    }

    fn lookup_e(&self, stmt: Statement) -> String {
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
#[derive(Debug)]
enum LabelData {
    Ap(Assertion),
    Ef(Statement),
}
struct MM {
    fs: FrameStack,
    labels: HashMap<String, LabelEntry>,
    begin_label: Option<String>,
    stop_label: Option<String>,
}
use crate::LabelData::Ef;
use crate::LabelData::Ap;

impl MM {
    fn new(begin_label: Option<String>, stop_label: Option<String>) -> MM {
        MM {
            fs: FrameStack::default(),
            labels: HashMap::new(),
            begin_label,
            stop_label ,
        }
    }

    fn read(&mut self, toks: &mut Tokens) {
        println!("Starting function read");
        self.fs.push();
        let mut label: Option<String> = None;
        let mut tok = toks.read_comment();
        println!("In MM read, found token to be {:?}", tok);
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
                    let label_u = label.expect("$f must have a label");
                    if stat.len() != 2 {
                        panic!("$f must have length 2");
                    }

                    println!("{} $f {} {} $.", label_u, stat[0].clone(), stat[1].clone());
                    self.fs.add_f(stat[1].clone(), stat[0].clone(), label_u.to_string());
                    let data = Ef(vec![stat[0].clone(), stat[1].clone()]);
                    self.labels.insert(label_u.to_string(), ("$f".to_string(), data));
                    label = None;
                }
                Some("$a") => {
                    let label_u = label.expect("$a must have a label");
                    match &self.stop_label {
                        Some(a) if a == &label_u => {
                            std::process::exit(0)
                        }
                        _ => {}
                    }

                    let data = Ap(self.fs.make_assertion(toks.readstat()));
                    self.labels.insert(label_u.to_string(), ("$a".to_string(), data));
                    label = None;
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
                    if label == self.stop_label {
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
                Some(x) if !x.starts_with('$') =>  {
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

    fn apply_subst(&self, stat: &Statement, subst: &HashMap<String, Statement> ) -> Statement {
        let mut result = vec![];

        for tok in stat {
            if subst.contains_key(tok.as_str()) {
                result.extend(subst[tok.as_str()].clone()); //very bad again
            } else {
                result.push(tok.to_string());
            }
        }
        result
    }

    fn find_vars(&self, stat: &Statement) -> Vec<String>{
        let mut vars: Vec<String> = vec!();
        for x in stat {
            if !vars.contains(x) && self.fs.lookup_v(x) {
                vars.push(x.to_owned());
            }
        }

        vars
    }

    fn decompress_proof(&mut self, stat: Statement, proof: Vec<String>) -> Vec<String> {

        let Assertion { dvs: _dm, f_hyps: mand_hyp_stmnts, e_hyps: hype_stmnts, stat: _ }  = self.fs.make_assertion(stat);

        let mand_hyps = mand_hyp_stmnts.iter().map(|(_k, v)| self.fs.lookup_f(v.to_string()));

        let hyps = hype_stmnts.iter().map(|s| self.fs.lookup_e(s.to_vec()));

        let mut labels: Vec<String> = mand_hyps.chain(hyps).collect();

        let hyp_end = labels.len();

        let ep = proof.iter().position(|x| x == ")").expect("Failed to find matching parthesis");

        labels.extend((&proof[1..ep]).iter().cloned());

        let compressed_proof = proof[ep + 1..].join("");

        println!("Labels {:?}", labels);
        println!("proof {}", compressed_proof);

        let mut proof_ints : Vec<i32> = vec!();
        let mut cur_int = 0;


        for ch in compressed_proof.chars() {
            if ch == 'Z' {
                proof_ints.push(-1); //change this to option instead of this hack
            } else if ('A'..='T').contains(&ch) {
                cur_int = 20 * cur_int + (ch as i32 - 'A' as i32 + 1) as i32;
                proof_ints.push(cur_int - 1);
                cur_int = 0;
            } else if ('U'..='Y').contains(&ch) {
                cur_int = 5 * cur_int + (ch as i32 - 'U' as i32 + 1) as i32;
            }
        }

        println!("proof_ints: {:?}", proof_ints);

        let label_end = labels.len();

        let mut decompressed_ints = vec!();
        let mut subproofs = vec!();
        let mut prev_proofs : Vec<Vec<i32>>= vec!();

        for pf_int in &proof_ints {
            let pf_int = *pf_int;
            if pf_int == -1 {
                subproofs.push(prev_proofs.last().unwrap().clone());
            } else if 0 <= pf_int && pf_int < hyp_end as i32 {
                prev_proofs.push(vec![pf_int]);
            } else if hyp_end <= pf_int as usize  && (pf_int as usize) < label_end {
                decompressed_ints.push(pf_int);

                let step = &self.labels[&labels[pf_int as usize]];


                let (_step_type, step_data) = step;

                match step_data {
                    Ap(Assertion {dvs : _sd, f_hyps: svars, e_hyps: shyps, stat: _sresult}) => {
                        let nhyps = shyps.len() + svars.len();

                        let new_prevpf : Vec<i32>;
                        if nhyps != 0 {
                            let new_index = prev_proofs.len() - nhyps;
                            new_prevpf = prev_proofs[(new_index)..].iter().flatten().copied().chain(std::iter::once(pf_int)).collect();
                            prev_proofs = prev_proofs[..new_index].to_vec();
                        } else {
                            new_prevpf = vec![pf_int];
                        }
                        prev_proofs.push(new_prevpf)
                    }
                    _ => {prev_proofs.push(vec![pf_int])}
                }
            } else if label_end <= pf_int as usize {
                let pf = &subproofs[pf_int as usize - label_end];
                println!("expanded subpf {:?}", pf);
                decompressed_ints.extend(pf);
                prev_proofs.push(pf.to_vec());
            }
        }

        println!("decompressed ints: {:?}", decompressed_ints);

        return decompressed_ints.iter().map(|i| labels[*i as usize].clone()).collect(); //fix the clone

    }

    fn verify(&mut self, _stat_label: String, stat: Statement, mut proof: Vec<String>) {
        let mut stack : Vec<Statement> = vec!();
        let _stat_type = stat[0].clone();
        if proof[0] == "(" {
            proof = self.decompress_proof(stat.clone(), proof);
        }

        for label in proof {
            let (_steptyp, stepdat) = &self.labels[&label];
            println!("{:?} : {:?}", label, self.labels[&label]);

            match stepdat {
                Ap(Assertion {dvs: distinct, f_hyps: mand_var, e_hyps: hyp, stat: result}) => {
                    println!("{:?}", stepdat);
                    let npop = mand_var.len() + hyp.len();
                    let sp = stack.len() - npop;
                    if stack.len() < npop {
                        panic!("stack underflow")
                    }
                    let mut sp = sp as usize;
                    let mut subst = HashMap::<String, Statement>::new();

                    for (k, v) in mand_var {
                        let entry: Statement = stack[sp].clone();

                        if &entry[0] != k {
                            panic!("stack entry doesn't match mandatry var hypothess");
                        }

                        subst.insert(v.to_string(), entry[1..].to_vec());
                        sp += 1;
                    }
                    println!("subst: {:?}", subst);

                    for (x, y) in distinct {
                        println!("dist {:?} {:?} {:?} {:?}", x, y, subst[x], subst[y]);
                        let x_vars = self.find_vars(&subst[x]);
                        let y_vars = self.find_vars(&subst[y]);

                        println!("V(x) = {:?}", x_vars);
                        println!("V(y) = {:?}", y_vars);

                        for x in &x_vars {
                            for y in &y_vars {
                                if x == y || !self.fs.lookup_d(x.to_string(), y.to_string()) {
                                    panic!("Disjoint violation");
                                }
                            }
                        }

                        for h in hyp {
                            let entry = &stack[sp];
                            let subst_h = self.apply_subst(&h.to_vec(), &subst);
                            if entry != &subst_h {
                                panic!("Stack entry doesn't match hypothesis")
                            }
                            sp += 1;
                        }

                        stack.drain(stack.len() - npop..);
                        stack.push(self.apply_subst(result, &subst));

                    }
                }
                Ef(x) => {
                    stack.push(x.to_vec());
                },
            }
            println!("st: {:?}", stack);

        }
        if stack.len() != 1 {
            panic!("stack has anentry greater than >1 at end")
        }
        if stack[0] != stat {
            panic!("assertion proved doesn't match ")
        }
    }

    fn dump(&mut self) {
        println!("{:?}", self.labels);
    }
}
fn main() {
    println!("Starting proof verification");

    let args : Vec<String> = std::env::args().collect();

    println!("Got cmd argumnets {:?}", args);

    let mut mm = MM::new(args.get(2).cloned(), args.get(3).cloned());

    let file = File::open(args[1].clone()).expect("Failed to find file");
    println!("Found file name {:?}", args[1]);
    mm.read(&mut Tokens::new(BufReader::new(file)));
}
