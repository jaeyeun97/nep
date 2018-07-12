use super::Terminal;
use super::termion::event::Key;
use std::boxed::Box;
use std::io;
use std::vec::Vec;

pub enum BindingMember {
    BindingKey(Key),
    NumericParameter(u32),
    StringParameter(String),
}

pub struct Binding {
    members: Vec<BindingMember>,
    action: Box<Fn(&Terminal, Vec<BindingMember>)>,
}

impl Binding {
    pub fn new(
        members: Vec<BindingMember>,
        action: Box<Fn(&Terminal, Vec<BindingMember>)>,
    ) -> Binding {
        Binding {
            members: members,
            action: action,
        }
    }

    fn matches(&self, &input: &Vec<Key>) -> bool {
        true
    }

    fn to_binding(&self, &input: &Vec<Key>) -> Vec<BindingMember> {
        // TODO detect numeric parameter matches etc.
        let mut detected: Vec<BindingMember> = Vec::new();
        for key in input.iter() {
            detected.push(BindingMember::BindingKey(*key));
        }

        detected
    }

    pub fn consume(&self, &terminal: &Terminal, &input: &Vec<Key>) -> Option<()> {
        if !self.matches(&input) {
            Option::None
        } else {
            Option::Some((*self.action)(&terminal, self.to_binding(&input)))
        }
    }
}

pub struct Mode {
    bindings: Vec<Box<Binding>>,
}

impl Mode {
    pub fn new(bindings: Vec<Box<Binding>>) -> Mode {
        Mode { bindings: bindings }
    }

    pub fn consume(&self, &terminal: &Terminal, &input: &Vec<Key>) -> Option<()> {
        for binding in self.bindings.iter() {
            match (*binding).consume(&terminal, &input) {
                Option::Some(x) => return Option::Some(x),
                Option::None => continue,
            };
        }

        Option::None
    }
}
