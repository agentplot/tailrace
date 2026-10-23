//! The one script every fake shares (design D7): faults a test sets on an
//! operation, refused or unreadable with the source's words, once or
//! until healed; and the record of every operation asked, in order, with
//! what it was given and what it answered.

use std::cell::RefCell;
use std::rc::Rc;

use serde::Serialize;
use serde_json::Value;
use tailrace_seams::{Answer, Not, Seam, Source, Why};

/// The profile name every fake answers as.
pub const PROFILE: &str = "fake";

/// What a faulted operation answers, with the words the test gives as the
/// source's own, which it answers verbatim (58, 59).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    Refused(String),
    Unreadable(String),
}

/// How long a fault holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Times {
    /// The next asking only.
    Once,
    /// Every asking until [`Fake::heal`].
    UntilHealed,
}

/// One operation asked of a fake: which, what it was given and what it
/// answered, each as its machine-readable form (57).
#[derive(Clone, Debug, PartialEq)]
pub struct Call<Op> {
    pub op: Op,
    pub given: Value,
    pub answer: Answer<Value>,
}

/// A fake's faults and its record.
pub struct Script<Op> {
    source: Source,
    faults: Vec<(Op, Fault, Times)>,
    calls: Vec<Call<Op>>,
}

impl<Op: Copy + PartialEq> Script<Op> {
    pub fn new(seam: Seam) -> Self {
        Script {
            source: Source::new(seam, PROFILE),
            faults: Vec::new(),
            calls: Vec::new(),
        }
    }

    /// The seam and profile every answer of this fake names.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// `op` answers `fault` from its next asking, for as long as `times`
    /// says; faults set on one operation answer in the order set.
    pub fn fault(&mut self, op: Op, fault: Fault, times: Times) {
        self.faults.push((op, fault, times));
    }

    /// Every fault set on `op` lifted.
    pub fn heal(&mut self, op: Op) {
        self.faults.retain(|(o, _, _)| *o != op);
    }

    /// Every operation asked, in order.
    pub fn calls(&self) -> &[Call<Op>] {
        &self.calls
    }

    /// `op` asked: the fault set on it, answered before `act` touches
    /// `state`, so a refused put holds nothing; otherwise what `act`
    /// answers. Recorded either way. `what` is what the operation reads,
    /// named in an unreadable answer.
    pub fn ask<S, T: Serialize>(
        &mut self,
        state: &mut S,
        op: Op,
        given: Value,
        what: impl Into<String>,
        act: impl FnOnce(&mut S, &Source) -> Answer<T>,
    ) -> Answer<T> {
        let answer = match self.take(op) {
            Some(Fault::Refused(words)) => Err(Not::refused(&self.source, Why::Said, Some(words))),
            Some(Fault::Unreadable(words)) => Err(Not::unreadable(&self.source, what, Some(words))),
            None => act(state, &self.source),
        };
        self.calls.push(Call {
            op,
            given,
            answer: match &answer {
                Ok(value) => Ok(serde_json::to_value(value).expect("a seam's answer serializes")),
                Err(not) => Err(not.clone()),
            },
        });
        answer
    }

    fn take(&mut self, op: Op) -> Option<Fault> {
        let i = self.faults.iter().position(|(o, _, _)| *o == op)?;
        match self.faults[i].2 {
            Times::Once => Some(self.faults.remove(i).1),
            Times::UntilHealed => Some(self.faults[i].1.clone()),
        }
    }
}

/// A fake: its seam's state and its script, shared by every clone, so a
/// test keeps one handle and binds another (design D7).
pub struct Fake<S, Op>(Rc<RefCell<(S, Script<Op>)>>);

impl<S, Op> Clone for Fake<S, Op> {
    fn clone(&self) -> Self {
        Fake(Rc::clone(&self.0))
    }
}

impl<S, Op: Copy + PartialEq> Fake<S, Op> {
    pub(crate) fn with(seam: Seam, state: S) -> Self {
        Fake(Rc::new(RefCell::new((state, Script::new(seam)))))
    }

    /// `op` answers `fault` from its next asking, for as long as `times`
    /// says.
    pub fn fault(&self, op: Op, fault: Fault, times: Times) {
        self.0.borrow_mut().1.fault(op, fault, times);
    }

    /// Every fault set on `op` lifted.
    pub fn heal(&self, op: Op) {
        self.0.borrow_mut().1.heal(op);
    }

    /// Every operation asked of this fake through any handle, in order.
    pub fn calls(&self) -> Vec<Call<Op>> {
        self.0.borrow().1.calls().to_vec()
    }

    pub(crate) fn ask<T: Serialize>(
        &self,
        op: Op,
        given: Value,
        what: impl Into<String>,
        act: impl FnOnce(&mut S, &Source) -> Answer<T>,
    ) -> Answer<T> {
        let (state, script) = &mut *self.0.borrow_mut();
        script.ask(state, op, given, what, act)
    }

    /// The state, for a scripting method to set.
    pub(crate) fn state(&self) -> std::cell::RefMut<'_, S> {
        std::cell::RefMut::map(self.0.borrow_mut(), |(state, _)| state)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Op {
        Put,
        Read,
    }

    type Counter = Fake<u32, Op>;

    fn counter() -> Counter {
        Fake::with(Seam::Store, 0)
    }

    fn put(c: &Counter) -> Answer<u32> {
        c.ask(Op::Put, json!({}), "the count", |n, _| {
            *n += 1;
            Ok(*n)
        })
    }

    fn read(c: &Counter) -> Answer<u32> {
        c.ask(Op::Read, json!({}), "the count", |n, _| Ok(*n))
    }

    #[test]
    fn a_fault_scripted_once_answers_once_and_then_heals() {
        let c = counter();
        c.fault(Op::Put, Fault::Refused("no room".into()), Times::Once);
        assert_eq!(
            put(&c),
            Err(Not::refused(
                &Source::new(Seam::Store, "fake"),
                Why::Said,
                Some("no room".into())
            ))
        );
        assert_eq!(read(&c), Ok(0), "a refused put held nothing");
        assert_eq!(put(&c), Ok(1));
        assert_eq!(put(&c), Ok(2));
    }

    #[test]
    fn a_fault_scripted_until_healed_answers_every_time() {
        let c = counter();
        c.fault(
            Op::Read,
            Fault::Unreadable("connection refused".into()),
            Times::UntilHealed,
        );
        for _ in 0..3 {
            assert_eq!(
                read(&c),
                Err(Not::unreadable(
                    &Source::new(Seam::Store, "fake"),
                    "the count",
                    Some("connection refused".into())
                ))
            );
        }
        assert_eq!(put(&c), Ok(1), "a fault on one operation leaves the others");
        c.heal(Op::Read);
        assert_eq!(read(&c), Ok(1));
    }

    #[test]
    fn faults_on_one_operation_answer_in_the_order_set() {
        let c = counter();
        c.fault(Op::Put, Fault::Refused("first".into()), Times::Once);
        c.fault(Op::Put, Fault::Unreadable("second".into()), Times::Once);
        assert!(matches!(put(&c), Err(Not::Refused(r)) if r.said.as_deref() == Some("first")));
        assert!(matches!(put(&c), Err(Not::Unreadable(u)) if u.said.as_deref() == Some("second")));
        assert_eq!(put(&c), Ok(1));
    }

    #[test]
    fn every_call_is_recorded_in_order_through_every_handle() {
        let kept = counter();
        let bound = kept.clone();
        bound.fault(Op::Read, Fault::Refused("busy".into()), Times::Once);
        put(&bound).unwrap();
        read(&bound).unwrap_err();
        read(&kept).unwrap();
        let calls = kept.calls();
        assert_eq!(
            calls.iter().map(|c| c.op).collect::<Vec<_>>(),
            [Op::Put, Op::Read, Op::Read]
        );
        assert_eq!(calls[0].answer, Ok(json!(1)));
        assert!(matches!(&calls[1].answer, Err(Not::Refused(_))));
        assert_eq!(calls[2].answer, Ok(json!(1)));
        assert_eq!(*kept.state(), 1);
    }
}
