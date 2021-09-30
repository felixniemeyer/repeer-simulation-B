use rand::rngs::ThreadRng;
use rand::Rng; 
use core::fmt;
use std::collections::HashMap;

struct GameParams {
    borrower_defect_payout: f64, 
    borrower_coop_payout: f64, 
    lender_defect_payout: f64, 
    lender_coop_payout: f64
}

const GP: GameParams = GameParams {
    borrower_defect_payout: 6., // steals the device
    borrower_coop_payout: 3., // uses the device
    lender_defect_payout: -7., // loses the device
    lender_coop_payout: -1., // lending effort + device wear
};

type BorrowerAction = bool; 
const ACCEPT: BorrowerAction = true; 
const REJECT: BorrowerAction = false; 

type LenderAction = bool; 
const COOP: BorrowerAction = true; 
// const DEFECT: BorrowerAction = false; 

trait Strategy : ToString {
    fn accept_or_reject_request(&mut self, borrower: usize) -> BorrowerAction; 
    fn notify_about_rejection(&mut self, lender: usize); 
    fn coop_or_defect(&mut self, lender: usize) -> LenderAction; 
    fn notify_coop_or_defect(&mut self, borrower: usize, coop: BorrowerAction); 
    fn get_type(&self) -> String;
    fn clone(&self) -> Box<dyn Strategy>; 
}

struct ReputationTracker {
    reputations: HashMap<usize, f64>, 
    optimistic: bool, 
}

impl ReputationTracker {
    fn new(optimistic: bool) -> ReputationTracker {
        ReputationTracker {
            reputations: HashMap::<usize, f64>::new(), 
            optimistic, 
        }
    }
}

impl Strategy for ReputationTracker {
    fn accept_or_reject_request(&mut self, borrower: usize) -> BorrowerAction {
        match self.reputations.get_mut(&borrower) {
            Some(r) => {
                if *r > 0.0 || (*r == 0.0 && self.optimistic) { 
                    ACCEPT
                } else { 
                    REJECT 
                }
            }, 
            None => {
                if self.optimistic {
                    ACCEPT
                } else {
                    REJECT
                }
            }
        }
    }
    fn notify_about_rejection(&mut self, _lender: usize) {
    }
    fn coop_or_defect(&mut self, lender: usize) -> LenderAction {
        match self.reputations.get_mut(&lender) {
            Some(r) => {
                *r += GP.borrower_coop_payout; 
                return COOP; 
            }, 
            None => {
                self.reputations.insert(lender, GP.borrower_coop_payout); 
                return COOP; 
            }
        }
    }
    fn notify_coop_or_defect(&mut self, borrower: usize, coop: BorrowerAction) {
        let penalty = if coop { 
            GP.lender_coop_payout 
        } else { 
            GP.lender_defect_payout 
        }; 
        match self.reputations.get_mut(&borrower) {
            Some(r) => {
                *r += penalty; 
            }, 
            None => {
                self.reputations.insert(borrower, penalty); 
            }
        }
    }
    fn get_type(&self) -> String { 
        "reputation tracker".into() 
    }
    fn clone(&self) -> Box<dyn Strategy> {
        Box::new( Self {
            reputations: self.reputations.clone(), 
            optimistic: self.optimistic
        })
    }
}

impl ToString for ReputationTracker {
    fn to_string(&self) -> String {
        format!("Strategy: {}; optimistic: {}", self.get_type(), self.optimistic)   
    }
}

struct RandomStrategy {
    rng: ThreadRng, 
    accept_prob: f32, 
    coop_prob: f32, 
    type_name: String
}

impl RandomStrategy {
    fn new(accept_prob: f32, coop_prob: f32, type_name: String) -> Self {
        Self {
            rng: rand::thread_rng(),
            accept_prob, 
            coop_prob, 
            type_name
        }
    }
}

impl Strategy for RandomStrategy {
    fn accept_or_reject_request(&mut self, _borrower: usize) -> BorrowerAction {
        self.rng.gen::<f32>() <= self.accept_prob
    }

    fn notify_about_rejection(&mut self, _lender: usize) {
    }

    fn coop_or_defect(&mut self, _lender: usize) -> LenderAction {
        self.rng.gen::<f32>() <= self.coop_prob
    }

    fn notify_coop_or_defect(&mut self, _borrower: usize, _coop: BorrowerAction) {
    }

    fn get_type(&self) -> String {
        self.type_name.clone()
    }
    fn clone(&self) -> Box<dyn Strategy> {
        Box::new(
            Self::new(
                self.accept_prob, 
                self.coop_prob, 
                self.type_name.clone()
            )
        ) 
    }
}

impl ToString for RandomStrategy {
    fn to_string(&self) -> String {
        format!(
            "Strategy: {}; accept_prop: {}, coop_prob: {}", 
            self.type_name, 
            self.accept_prob, 
            self.coop_prob
        )
    }
}

// struct Alternating {
//     last_response: BorrowerAction
// }
// impl ResponseStrategy for Alternating {
//     fn evaluate_request(&mut self, _borrower: usize) -> BorrowerAction { 
//         self.last_response = !self.last_response;
//         self.last_response
//     }
//     fn get_type(&self) -> &str { "rejecter" }
// }

// struct Mirror {
//     last_action: HashMap<usize, f64>, 
//     optimistic: bool // trusts initially? 
// }
// 
// impl Player for Mirror {
//     fn accept_or_reject_request(&mut self, 
// }

struct Agent {
    pub strategy: Box<dyn Strategy>, 
    energy: f64, 
    id: usize
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}|{}|{}", self.id, self.energy, self.strategy.get_type())
    }
}

type AgentDefinition = (fn() -> Box<dyn Strategy>, usize);

fn main() {
    fn reptrack() -> Box<dyn Strategy> { Box::new(ReputationTracker::new(true)) }
    fn never_accept_always_defect() -> Box<dyn Strategy> { 
        Box::new(RandomStrategy::new(0.0, 0.0, "never accept, always defect".into()))
    }
    fn random() -> Box<dyn Strategy> { 
        Box::new(RandomStrategy::new(0.5, 0.5, "random 50/50".into())) 
    }
    let mut agents = gen_agents(vec![
        (reptrack, 64), 
        (never_accept_always_defect, 32), 
        (random, 32), 
    ]);

    println!("{:?}", agents);

    simulate(&mut agents, 30);
}

fn gen_agents(agent_definitions: Vec<AgentDefinition>) -> Vec<Agent> {
    let mut agents: Vec<Agent> = vec![];

    let mut last_id = 0; 
    let mut id = 0; 
    for agent_def in agent_definitions {
        for i in 0..agent_def.1 {
            id = i + last_id; 
            agents.push(Agent {
                strategy: agent_def.0(), 
                energy: 256., 
                id
            }) 
        }
        last_id = id; 
    }

    agents
}

fn simulate(agents: &mut Vec<Agent>, rounds: i32) {
    for round in 0..rounds {
        println!("Round {}.", round); 
        report(agents); 
        for i in 1..agents.len() {
            let (left, right) = agents.split_at_mut(i); 
            let alice = left.last_mut().unwrap();
            for bob in right.iter_mut() {
                encounter(alice, bob); 
                encounter(bob, alice); 
            }
        }
        agents.retain(|agent| agent.energy > 0.); 
    }
}

fn report(agents: &Vec<Agent>) {
    // println!("simulating agents: {:?}", agents); 
    let mut count: HashMap<String, i32> = HashMap::new(); 
    let mut sum: HashMap<String, f64> = HashMap::new(); 

    for agent in agents.iter() {
        let st = agent.strategy.get_type();
        match count.get_mut(&st.to_string()) {
            Some(n) => { *n += 1; }, 
            None => { count.insert(st.to_string(), 1); }
        };
        match sum.get_mut(&st.to_string()) {
            Some(energy) => { *energy += agent.energy; }, 
            None => { sum.insert(st.to_string(), agent.energy); }
        };
    }

    let mut keys = count.keys().collect::<Vec<&String>>(); 
    keys.sort(); 

    for strategy in keys.iter() {
        let c = count.get(*strategy).unwrap();
        println!("{}:", strategy); 
        println!(" - count: {}", c); 
        match sum.get(*strategy) {
            Some(s) => {
                println!(" - mean energy: {:.2}", s / (*c as f64))
            }, 
            None => {}
        };
    }

    println!()
}

fn encounter(lender: &mut Agent, borrower: &mut Agent) {
    if lender.strategy.accept_or_reject_request(borrower.id) == ACCEPT {
        let coop = borrower.strategy.coop_or_defect(lender.id);
        lender.strategy.notify_coop_or_defect(borrower.id, coop); 
        if coop == COOP {
            lender.energy += GP.lender_coop_payout; 
            borrower.energy += GP.borrower_coop_payout; 
        } else {
            lender.energy += GP.lender_defect_payout; 
            borrower.energy += GP.borrower_defect_payout; 
        }
    } else {
        borrower.strategy.notify_about_rejection(lender.id); 
    }
}
