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
pub(crate) enum CaptureGroupInstruction {
    Start(u64),
    End(u64),
    None,
}

impl CaptureGroupInstruction {
    pub(crate) fn to_label(&self) -> String {
        match self {
            Self::Start(id) => format!("CapS[{}]", id),
            Self::End(id) => format!("CapE[{}]", id),
            Self::None => String::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Transition {
    pub(crate) from_state: u64,
    pub(crate) to_state: u64,
    pub(crate) cond: Cond,
    pub(crate) max_use: Option<u64>,
    pub(crate) capture_group_ins: CaptureGroupInstruction,
}

impl Transition {
    pub(crate) fn new_full(
        from_state: u64,
        to_state: u64,
        cond: Cond,
        max_use: Option<u64>,
        capture_group_ins: CaptureGroupInstruction,
    ) -> Self {
        Self {
            from_state,
            to_state,
            cond,
            max_use,
            capture_group_ins,
        }
    }

    pub(crate) fn new_cond(from_state: u64, to_state: u64, cond: Cond) -> Self {
        Self {
            from_state,
            to_state,
            cond,
            max_use: None,
            capture_group_ins: CaptureGroupInstruction::None,
        }
    }

    pub(crate) fn new(from_state: u64, to_state: u64) -> Self {
        Self {
            from_state,
            to_state,
            cond: Cond::None,
            max_use: None,
            capture_group_ins: CaptureGroupInstruction::None,
        }
    }

    fn to_label(&self) -> String {
        let mut parts: Vec<String> = vec![self.cond.to_label()];

        if let Some(v) = self.max_use {
            parts.push(format!("(max {})", v));
        }

        let capture_part = self.capture_group_ins.to_label();
        if !capture_part.is_empty() {
            parts.push(capture_part);
        }

        parts.join(" ")
    }
}

#[allow(dead_code)]
pub(crate) fn create_dot_file_from_transitions(transitions: &Vec<Transition>) {
    let mut f = File::create("./state_machine.dot").unwrap();

    f.write_all(b"digraph {{\n").unwrap();

    for (i, tr) in transitions.iter().enumerate() {
        f.write_all(
            format!(
                "\t{} -> {} [label=\"#{}\n{}\"]\n",
                state_id_to_label(tr.from_state),
                state_id_to_label(tr.to_state),
                i,
                tr.to_label()
            )
            .as_bytes(),
        )
        .unwrap();
    }

    f.write_all(b"}}\n").unwrap();
}
