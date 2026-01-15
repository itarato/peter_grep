use std::{fs::File, io::Write};

use crate::cond::Cond;

fn state_id_to_label(id: u64) -> String {
    match id {
        0 => "Start".to_string(),
        1 => "End".to_string(),
        other => other.to_string(),
    }
}

#[derive(Debug)]
pub(crate) struct Transition {
    pub(crate) from_state: u64,
    pub(crate) to_state: u64,
    pub(crate) cond: Cond,
    pub(crate) max_use: Option<u64>,
}

impl Transition {
    fn to_label(&self) -> String {
        match self.max_use {
            Some(v) => format!("{} (max {})", self.cond.to_label(), v),
            None => self.cond.to_label(),
        }
    }
}

#[allow(dead_code)]
pub(crate) fn create_dot_file_from_transitions(transitions: &Vec<Transition>) {
    let mut f = File::create("./state_machine.dot").unwrap();

    f.write_all(b"digraph {{\n").unwrap();

    for tr in transitions {
        f.write_all(
            format!(
                "\t{} -> {} [label=\"{}\"]\n",
                state_id_to_label(tr.from_state),
                state_id_to_label(tr.to_state),
                tr.to_label()
            )
            .as_bytes(),
        )
        .unwrap();
    }

    f.write_all(b"}}\n").unwrap();
}
