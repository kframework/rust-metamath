use std::cmp::max;
use std::cmp::min;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::rc::Rc;

#[derive(Debug)]
struct Tokens {
    lines_buffer: Vec<BufReader<File>>,
    token_buffer: Vec<String>,
    imported_files: HashSet<String>,
}

//since statement may be used multiple times when applying substitution
// use Rc
type Statement = Rc<[LanguageToken]>; //may be better to newtype this but I guess it works for now

type Proof = Vec<Label>; //I don't think a proof is used multiple times
type Label = Rc<str>;
impl Tokens {
    fn new(lines: BufReader<File>) -> Tokens {
        Tokens {
            lines_buffer: vec![lines],
            token_buffer: vec![],
            imported_files: HashSet::new(),
        }
    }
    fn read(&mut self) -> Option<String> {
        // println!("inside read function with state {:?}", self);
        while self.token_buffer.is_empty() {
            //println!("Buffer is empty, refilling");
            let mut line = String::new();
            // pretend this succeeeds
            let result = self.lines_buffer.last_mut().unwrap().read_line(&mut line);
            // println!("Read line: {}", line);

            match result {
                Ok(num) if num > 0 => {
                    // println!("Read {} lines ", num);
                    self.token_buffer = line.split_whitespace().map(|x| x.into()).collect();
                    self.token_buffer.reverse();
                }
                _ => {
                    // println!("Done with file");
                    self.lines_buffer.pop();
                    if self.lines_buffer.is_empty() {
                        return None;
                    }
                }
            }
            // println!("Created token buffer {:?}", self.token_buffer);
        }
        self.token_buffer.pop()
    }

    fn read_file(&mut self) -> Option<String> {
        // println!("reading file");

        let mut token = self.read();
        // println!("In read file found token {:?}", token);
        while let Some("$[") = token.as_deref() {
            let filename = self.read().expect("Couldn't find filename");
            let endbracket = self.read().expect("Coludn't find end bracket");

            // println!("In read file found filename: {:?}, endbracket: {:?}", filename, endbracket);
            if endbracket != "$]" {
                panic!("End bracket not found");
            }

            if !self.imported_files.contains(&filename) {
                // println!("Found new file {}", &filename);

                self.lines_buffer.push(BufReader::new(
                    File::open(filename.clone()).expect("Failed to open file"),
                ));
                self.imported_files.insert(filename);
            }
            token = self.read();
        }
        token
    }

    fn read_comment(&mut self) -> Option<String> {
        // println!("reading comment");

        loop {
            let mut token = self.read_file();
            // println!("In read comment: found token to be {:?}", token);
            match &token {
                None => return None,
                Some(x) if x == "$(" => loop {
                    match token.as_deref() {
                        Some("$)") => break,
                        _ => token = self.read(),
                    }
                },
                _ => return token,
            }
        }
    }

    fn readstat(&mut self) -> Statement {
        let mut stat: Vec<Rc<str>> = vec![];
        let mut token = self
            .read_comment()
            .expect("Failed to read token in read stat");

        // println!("In read stat, found token to be {:?}", token);
        while token != "$." {
            stat.push(token.into());
            token = self.read_comment().expect("EOF before $.");
        }
        stat.into()
    }
}

type LanguageToken = Rc<str>;

#[derive(Default, Debug)]
struct Frame {
    c: HashSet<LanguageToken>,
    v: HashSet<LanguageToken>,
    d: HashSet<(LanguageToken, LanguageToken)>, //maybe switch this give c and v different types
    f: Vec<(LanguageToken, LanguageToken)>,
    f_labels: HashMap<LanguageToken, Label>,
    e: Vec<Statement>,
    e_labels: HashMap<Statement, Label>,
}

#[derive(Default, Debug)]
struct FrameStack {
    list: Vec<Frame>,
}

#[derive(Debug)]
struct Assertion {
    dvs: HashSet<(LanguageToken, LanguageToken)>,
    f_hyps: VecDeque<(LanguageToken, LanguageToken)>,
    e_hyps: Vec<Statement>,
    stat: Statement,
}

impl FrameStack {
    fn push(&mut self) {
        self.list.push(Frame::default());
    }

    fn add_c(&mut self, token: LanguageToken) {
        let frame = &mut self.list.last_mut().unwrap();

        if frame.c.contains(&token) {
            panic!("Const already defined")
        }
        if frame.v.contains(&token) {
            panic!("consta elaryd defined as var in scope")
        }
        frame.c.insert(token);
    }

    fn add_v(&mut self, token: LanguageToken) {
        let frame = &mut self.list.last_mut().unwrap();

        if frame.c.contains(&token) {
            panic!("Variable already defined")
        }
        if frame.v.contains(&token) {
            panic!("Variable elaryd defined as var in scope")
        }
        frame.v.insert(token);
    }

    fn add_f(&mut self, var: LanguageToken, kind: LanguageToken, label: Label) {
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

    fn add_e(&mut self, stat: Statement, label: Label) {
        let frame = self.list.last_mut().unwrap();

        frame.e.push(stat.clone());
        frame.e_labels.insert(stat, label);
    }

    fn add_d(&mut self, stat: Statement) {
        let frame = self.list.last_mut().unwrap();
        //let mut product_vec = vec!();
        for x in stat.iter() {
            for y in stat.iter() {
                if x != y {
                    frame
                        .d
                        .insert((min(x.clone(), y.clone()), max(x.clone(), y.clone())));
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

    fn lookup_f(&self, var: LanguageToken) -> Label {
        // println!("lookup {}", var);
        let f = self
            .list
            .iter()
            .rev()
            .find(|frame| frame.f_labels.contains_key(&var))
            .unwrap();

        f.f_labels[&var].clone()
    }

    fn lookup_d(&mut self, x: LanguageToken, y: LanguageToken) -> bool {
        self.list.iter().rev().any(|fr| {
            fr.d.contains(&(min(x.clone(), y.clone()), max(x.clone(), y.clone())))
        })
    }

    fn lookup_e(&self, stmt: Statement) -> Label {
        let f = self
            .list
            .iter()
            .rev()
            .find(|frame| frame.e_labels.contains_key(&stmt))
            .expect("Bad e");

        f.e_labels[&stmt].clone()
    }

    fn make_assertion(&mut self, stat: Statement) -> Assertion {
        let _frame = self.list.last_mut().unwrap();

        let e_hyps: Vec<Statement> = self.list.iter().flat_map(|fr| fr.e.clone()).collect();

        let chained = e_hyps.iter().chain(std::iter::once(&stat));

        let mut mand_vars: HashSet<LanguageToken> = chained
            .flat_map(|x| x.iter())
            .filter(|tok| self.lookup_v(tok))
            .cloned()
            .collect(); //cloned should do a shallow copy

        // println!("ma: \n mand_vars: {:?}, ", mand_vars);

        let mut cartesian: HashSet<(LanguageToken, LanguageToken)> = HashSet::new();

        for x in mand_vars.iter() {
            for y in mand_vars.iter() {
                cartesian.insert((x.clone(), y.clone()));
            }
        }

        let dvs: HashSet<(LanguageToken, LanguageToken)> = self
            .list
            .iter()
            .flat_map(|fr| fr.d.intersection(&cartesian))
            .cloned()
            .collect();

        let mut f_hyps = VecDeque::new();
        self.list.iter().rev().for_each(|fr| {
            fr.f.iter().rev().for_each(|(v, k)| {
                if mand_vars.contains(v) {
                    f_hyps.push_front((k.clone(), v.clone()));
                    mand_vars.remove(v);
                }
            });
        });
        // println!("ma: \n dvs: {:?}, f: {:?}, e_hyps: {:?}, stat: {:?}", dvs, f_hyps, e_hyps, stat);

        Assertion {
            dvs,
            f_hyps,
            e_hyps,
            stat,
        }
    }
}

// first one is label type,

#[derive(Debug)]
enum LabelEntry {
    DollarA(Assertion),
    DollarP(Assertion),
    DollarE(Statement),
    DollarF(Statement),
}

struct MM {
    fs: FrameStack,
    labels: HashMap<Label, Rc<LabelEntry>>,
    begin_label: Option<String>,
    stop_label: Option<String>,
}

impl MM {
    fn new(begin_label: Option<String>, stop_label: Option<String>) -> MM {
        MM {
            fs: FrameStack::default(),
            labels: HashMap::new(),
            begin_label,
            stop_label,
        }
    }

    fn read(&mut self, toks: &mut Tokens) {
        // println!("Starting function read");
        self.fs.push();
        let mut label: Option<String> = None;
        let mut tok = toks.read_comment();
        // println!("In MM read, found token to be {:?}", tok);
        loop {
            match tok.as_deref() {
                Some("$}") => break,
                Some("$c") => {
                    for tok in toks.readstat().iter() {
                        self.fs.add_c(tok.clone());
                    }
                }
                Some("$v") => {
                    for tok in toks.readstat().iter() {
                        self.fs.add_v(tok.clone());
                    }
                }
                Some("$f") => {
                    let stat = toks.readstat();
                    let label_u: Label = label.expect("$f must have a label").into();
                    if stat.len() != 2 {
                        panic!("$f must have length 2");
                    }

                    // println!("{} $f {} {} $.", label_u, stat[0].clone(), stat[1].clone());
                    self.fs
                        .add_f(stat[1].clone(), stat[0].clone(), label_u.clone());
                    let data = LabelEntry::DollarF(Rc::new([stat[0].clone(), stat[1].clone()]));
                    self.labels.insert(label_u, Rc::new(data));
                    label = None;
                }
                Some("$a") => {
                    let label_u = label.expect("$a must have a label");
                    match &self.stop_label {
                        Some(a) if a == &label_u => std::process::exit(0),
                        _ => {}
                    }

                    let data = LabelEntry::DollarA(self.fs.make_assertion(toks.readstat()));
                    self.labels.insert(label_u.into(), Rc::new(data));
                    label = None;
                }

                Some("$e") => {
                    let label_u: Label = label.expect("e must have label").into();

                    let stat = toks.readstat();
                    self.fs.add_e(stat.clone(), label_u.clone());
                    let data = LabelEntry::DollarE(stat);
                    self.labels.insert(label_u.clone(), Rc::new(data));
                    label = None;
                }
                Some("$p") => {
                    let label_u = label.clone().expect("$p must have elabel");
                    if label == self.stop_label {
                        //could be rewritten better
                        std::process::exit(0);
                    }
                    let stat = toks.readstat();
                    let i = stat
                        .iter()
                        .position(|x| x.as_ref() == "$=")
                        .expect("Mmust have $=");
                    let proof = &stat[i + 1..].to_vec();
                    let stat = &stat[..i];

                    if self.begin_label.is_some() && &label_u == self.begin_label.as_ref().unwrap()
                    {
                        self.begin_label = None;
                    }
                    if self.begin_label.is_none() {
                        println!("verifying {}", label_u);
                        self.verify(label_u.clone(), stat.into(), proof.to_vec());
                    }
                    let data = LabelEntry::DollarP(self.fs.make_assertion(stat.into()));
                    self.labels.insert(label_u.into(), Rc::new(data));
                    label = None;
                }
                Some("$d") => {
                    self.fs.add_d(toks.readstat());
                }
                Some("${") => {
                    self.read(toks);
                }
                Some(x) if !x.starts_with('$') => {
                    label = tok;
                }
                Some(_) => {
                    // print!("tok {:?}", tok);
                }
                None => break,
            }
            tok = toks.read_comment();
        }
        self.fs.list.pop();
    }

    fn apply_subst(
        &self,
        stat: &Statement,
        subst: &HashMap<LanguageToken, Statement>,
    ) -> Statement {
        let mut result: Vec<LanguageToken> = vec![];

        for tok in stat.iter() {
            if subst.contains_key(tok.as_ref()) {
                result.extend(subst[tok.as_ref()].iter().cloned()); //the cloned shouldn't deep copy
            } else {
                result.push(tok.clone());
            }
        }
        result.into()
    }

    fn find_vars(&self, stat: Statement) -> Vec<LanguageToken> {
        let mut vars: Vec<LanguageToken> = vec![];
        for x in stat.iter() {
            if !vars.contains(x) && self.fs.lookup_v(x) {
                vars.push(x.to_owned());
            }
        }

        vars
    }

    fn decompress_proof(&mut self, stat: Statement, proof: Proof) -> Proof {
        //I should change the type system to differentiate betweene the different types of proofs
        // println!("Statement {:?}", stat);

        let Assertion {
            dvs: _dm,
            f_hyps: mand_hyp_stmnts,
            e_hyps: hype_stmnts,
            stat: _,
        } = self.fs.make_assertion(stat);
        // println!("mand_hyps_stmnts {:?}", mand_hyp_stmnts);

        let mand_hyps = mand_hyp_stmnts
            .iter()
            .map(|(_k, v)| self.fs.lookup_f(v.clone()));

        let hyps = hype_stmnts.iter().map(|s| self.fs.lookup_e(s.clone()));

        // println!("mand_hyps {:?}", mand_hyps);
        // println!("hyps {:?}", hyps);
        let mut labels: Vec<Label> = mand_hyps.chain(hyps).collect(); // contains both the mandatory hypotheses and the ones in the proof
                                                                      // println!("Labels {:?}", labels);

        let hyp_end = labels.len(); //when the mandatory hypotheses end

        let ep = proof
            .iter()
            .position(|x| x.as_ref() == ")")
            .expect("Failed to find matching parthesis");

        labels.extend((&proof[1..ep]).iter().cloned());

        let compressed_proof = proof[ep + 1..].join("");

        // println!("Labels {:?}", labels);
        // println!("proof {}", compressed_proof);

        let mut proof_indeces: Vec<Option<usize>> = vec![];
        let mut cur_index: usize = 0;

        for ch in compressed_proof.chars() {
            if ch == 'Z' {
                proof_indeces.push(None);
            } else if ('A'..='T').contains(&ch) {
                cur_index = 20 * cur_index + (ch as i32 - 'A' as i32 + 1) as usize;
                proof_indeces.push(Some(cur_index - 1));
                cur_index = 0;
            } else if ('U'..='Y').contains(&ch) {
                cur_index = 5 * cur_index + (ch as i32 - 'U' as i32 + 1) as usize;
            }
        }
        // println!("proof_ints: {:?}", proof_ints);

        let label_end = labels.len();
        // println!("labels: {:?}", labels);

        let mut decompressed_ints: Vec<usize> = vec![];
        type CompressedProof = Rc<[usize]>;
        let mut subproofs: Vec<CompressedProof> = vec![]; //stuff tagged  with Zs
        let mut prev_proofs: Vec<CompressedProof> = vec![];

        for pf_int in &proof_indeces {
            // println!("subproofs : {:?}", subproofs);
            // println!("pf_int: {:?}, label_end: {:?}", pf_int, label_end);
            match pf_int {
                None => {
                    subproofs.push(
                        prev_proofs
                            .last()
                            .expect("Error in decompressing proof, found unexpected Z")
                            .clone(),
                    );
                }
                Some(i) if *i < hyp_end => {
                    //mandatory hypothesis
                    prev_proofs.push(Rc::new([*i]));
                    decompressed_ints.push(*i);
                }

                Some(i) if hyp_end <= *i && *i < label_end => {
                    //one of the given labels in the proof
                    decompressed_ints.push(*i);

                    let label_name = &labels[*i];

                    let step_data = &self.labels[label_name];

                    match &**step_data {
                        //syntax doesn't look correct
                        LabelEntry::DollarA(Assertion {
                            dvs: _sd,
                            f_hyps: svars,
                            e_hyps: shyps,
                            stat: _sresult,
                        })
                        | LabelEntry::DollarP(Assertion {
                            dvs: _sd,
                            f_hyps: svars,
                            e_hyps: shyps,
                            stat: _sresult,
                        }) => {
                            // when we get things that take hypothesis, we have to include those
                            // in the list of previos proof,
                            let nhyps = shyps.len() + svars.len();

                            let new_index = prev_proofs.len() - nhyps;

                            // let new_prevpf;
                            // if nhyps != 0 {
                            //     let new_index = prev_proofs.len() - nhyps;

                            //     new_prevpf = prev_proofs[(new_index)..]
                            //         .iter()
                            //         .map(|x| x.iter())
                            //         .flatten()
                            //         .chain(std::iter::once(i));
                            //     prev_proofs = prev_proofs[..new_index].to_vec();
                            // } else {
                            //     new_prevpf = std::iter::once(i);
                            // }

                            if nhyps != 0 {
                                let mand_hyps: Vec<CompressedProof> =
                                    prev_proofs.drain(new_index..).collect(); // I tried putting this in oneb ig iterator but it didn't work

                                let new_prevpf = mand_hyps
                                    .iter()
                                    .flat_map(|x| x.iter())
                                    .chain(std::iter::once(i));

                                prev_proofs.push(new_prevpf.copied().collect())
                            } else {
                                prev_proofs.push(Rc::new([*i]));
                            }
                        }
                        _ => prev_proofs.push(Rc::new([*i])),
                    }
                }

                Some(i) if label_end <= *i => {
                    // println!("*i: {:?}, label_end: {:?}", pf_int, label_end);
                    let pf = &subproofs[(*i as usize) - label_end];
                    // println!("expanded subpf {:?}", pf);
                    decompressed_ints.extend(pf.iter());
                    prev_proofs.push(pf.clone());
                }

                _ => {
                    panic!("Bad compression")
                }
            }
        }

        return decompressed_ints
            .iter()
            .map(|i| labels[*i as usize].clone())
            .collect(); //fix the clone
    }

    fn verify(&mut self, stat_label: String, stat: Statement, mut proof: Proof) {
        let mut stack: Vec<Statement> = vec![];
        let _stat_type = stat[0].clone();
        if proof[0].as_ref() == "(" {
            println!("Starting decompression for {}", stat_label);
            proof = self.decompress_proof(stat.clone(), proof);
            println!("Finished decompression for {}", stat_label);
        }

        if proof.is_empty() {
            println!("Did not find proof for {}, skipping", stat_label);
            return;
        }

        for label in proof {
            let stepdat = &self.labels[&label];
            // println!("{:?} : {:?}", label, self.labels[&label]);

            match &**stepdat {
                LabelEntry::DollarA(Assertion {
                    dvs: distinct,
                    f_hyps: mand_var,
                    e_hyps: hyp,
                    stat: result,
                })
                | LabelEntry::DollarP(Assertion {
                    dvs: distinct,
                    f_hyps: mand_var,
                    e_hyps: hyp,
                    stat: result,
                }) => {
                    // println!("{:?}", stepdat);
                    // println!("stack: {:?}", stack);
                    let npop = mand_var.len() + hyp.len();
                    // println!("stacklength {:?}, ", stack.len());
                    let sp = stack.len() - npop;
                    // println!("npop {:?}, sp {:?}", npop, sp);
                    if stack.len() < npop {
                        panic!("stack underflow")
                    }
                    let mut sp = sp as usize;
                    let mut subst = HashMap::<LanguageToken, Statement>::new();

                    for (k, v) in mand_var {
                        let entry: Statement = stack[sp].clone();
                        // println!("Before checking if equal {:?} : {:?} with sp {:?}", &entry[0], k, sp);

                        if &entry[0] != k {
                            panic!("stack entry doesn't match mandatry var hypothess");
                        }

                        subst.insert(v.clone(), entry[1..].into());
                        sp += 1;
                    }
                    // println!("subst: {:?}", subst);

                    for (x, y) in distinct {
                        // println!("dist {:?} {:?} {:?} {:?}", x, y, subst[x], subst[y]);
                        let x_vars = self.find_vars(Rc::clone(&subst[x]));
                        let y_vars = self.find_vars(subst[y].clone());

                        // println!("V(x) = {:?}", x_vars);
                        // println!("V(y) = {:?}", y_vars);

                        for x in &x_vars {
                            for y in &y_vars {
                                if x == y || !self.fs.lookup_d(x.clone(), y.clone()) {
                                    panic!("Disjoint violation");
                                }
                            }
                        }
                    }
                    for h in hyp {
                        let entry = &stack[sp];
                        let subst_h = self.apply_subst(h, &subst);
                        if entry != &subst_h {
                            panic!("Stack entry doesn't match hypothesis")
                        }
                        sp += 1;
                    }

                    // println!("stack: {:?}", stack);
                    stack.drain(stack.len() - npop..);
                    // println!("stack: {:?}", stack);
                    stack.push(self.apply_subst(result, &subst));
                    // println!("stack: {:?}", stack);
                }
                LabelEntry::DollarF(x) | LabelEntry::DollarE(x) => {
                    stack.push(x.clone());
                }
            }
            // // // // println!("st: {:?}", stack);
        }
        if stack.len() != 1 {
            panic!("stack has anentry greater than >1 at end")
        }
        if stack[0] != stat {
            panic!("assertion proved doesn't match ")
        }
    }

    fn dump(&mut self) {
        // println!("{:?}", self.labels);
    }
}
fn main() {
    // println!("Starting proof verification");

    let args: Vec<String> = std::env::args().collect();

    // println!("Got cmd argumnets {:?}", args);

    let mut mm = MM::new(args.get(2).cloned(), args.get(3).cloned());

    let file = File::open(args[1].clone()).expect("Failed to find file");
    // println!("Found file name {:?}", args[1]);
    use std::time::Instant;
    let now = Instant::now();

    mm.read(&mut Tokens::new(BufReader::new(file)));
    mm.dump();
    let elapsed = now.elapsed();
    println!("Finished checking in {:.2?}", elapsed);
}
