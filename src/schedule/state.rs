use crate::{
    schedule::{
        RunCriteriaDescriptor, RunCriteriaDescriptorCoercion, RunCriteriaLabel, ShouldRun,
        SystemSet,
    },
    system::{ConfigurableSystem, In, IntoChainSystem, Local, Res, ResMut},
};
use std::{any::TypeId, fmt::Debug, hash::Hash};
use thiserror::Error;

pub trait StateData: Send + Sync + Clone + Eq + Debug + Hash + 'static {}
impl<T> StateData for T where T: Send + Sync + Clone + Eq + Debug + Hash + 'static {}

/// ### Stack based state machine
///
/// This state machine has four operations: Push, Pop, Set and Replace.
/// * Push pushes a new state to the state stack, pausing the previous state
/// * Pop removes the current state, and unpauses the last paused state
/// * Set replaces the active state with a new one
/// * Replace unwinds the state stack, and replaces the entire stack with a single new state
#[derive(Debug)]
pub struct State<T: StateData> {
    transition: Option<StateTransition<T>>,
    stack: Vec<T>,
    scheduled: Option<ScheduledOperation<T>>,
    end_next_loop: bool,
}

#[derive(Debug)]
enum StateTransition<T: StateData> {
    PreStartup,
    Startup,
    // The parameter order is always (leaving, entering)
    ExitingToResume(T, T),
    ExitingFull(T, T),
    Entering(T, T),
    Resuming(T, T),
    Pausing(T, T),
}

#[derive(Debug)]
enum ScheduledOperation<T: StateData> {
    Set(T),
    Replace(T),
    Pop,
    Push(T),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum StateCallback {
    Update,
    InactiveUpdate,
    InStackUpdate,
    Enter,
    Exit,
    Pause,
    Resume,
}

impl StateCallback {
    fn into_label<T>(self, state: T) -> StateRunCriteriaLabel<T>
    where
        T: StateData,
    {
        StateRunCriteriaLabel(state, self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct StateRunCriteriaLabel<T>(T, StateCallback);
impl<T> RunCriteriaLabel for StateRunCriteriaLabel<T>
where
    T: StateData,
{
    fn dyn_clone(&self) -> Box<dyn RunCriteriaLabel> {
        Box::new(self.clone())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct DriverLabel(TypeId);
impl RunCriteriaLabel for DriverLabel {
    fn dyn_clone(&self) -> Box<dyn RunCriteriaLabel> {
        Box::new(self.clone())
    }
}

impl DriverLabel {
    fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

impl<T> State<T>
where
    T: StateData,
{
    pub fn on_update(s: T) -> RunCriteriaDescriptor {
        (|state: Res<State<T>>, pred: Local<Option<T>>| {
            state.stack.last().unwrap() == pred.as_ref().unwrap() && state.transition.is_none()
        })
        .config(|(_, pred)| *pred = Some(Some(s.clone())))
        .chain(should_run_adapter::<T>)
        .after(DriverLabel::of::<T>())
        .label_discard_if_duplicate(StateCallback::Update.into_label(s))
    }

    pub fn on_inactive_update(s: T) -> RunCriteriaDescriptor {
        (|state: Res<State<T>>, mut is_inactive: Local<bool>, pred: Local<Option<T>>| match &state
            .transition
        {
            Some(StateTransition::Pausing(ref relevant, _))
            | Some(StateTransition::Resuming(_, ref relevant)) => {
                if relevant == pred.as_ref().unwrap() {
                    *is_inactive = !*is_inactive;
                }
                false
            }
            Some(_) => false,
            None => *is_inactive,
        })
        .config(|(_, _, pred)| *pred = Some(Some(s.clone())))
        .chain(should_run_adapter::<T>)
        .after(DriverLabel::of::<T>())
        .label_discard_if_duplicate(StateCallback::InactiveUpdate.into_label(s))
    }

    pub fn on_in_stack_update(s: T) -> RunCriteriaDescriptor {
        (|state: Res<State<T>>, mut is_in_stack: Local<bool>, pred: Local<Option<T>>| match &state
            .transition
        {
            Some(StateTransition::Entering(ref relevant, _))
            | Some(StateTransition::ExitingToResume(_, ref relevant)) => {
                if relevant == pred.as_ref().unwrap() {
                    *is_in_stack = !*is_in_stack;
                }
                false
            }
            Some(StateTransition::ExitingFull(_, ref relevant)) => {
                if relevant == pred.as_ref().unwrap() {
                    *is_in_stack = !*is_in_stack;
                }
                false
            }
            Some(StateTransition::Startup) => {
                if state.stack.last().unwrap() == pred.as_ref().unwrap() {
                    *is_in_stack = !*is_in_stack;
                }
                false
            }
            Some(_) => false,
            None => *is_in_stack,
        })
        .config(|(_, _, pred)| *pred = Some(Some(s.clone())))
        .chain(should_run_adapter::<T>)
        .after(DriverLabel::of::<T>())
        .label_discard_if_duplicate(StateCallback::InStackUpdate.into_label(s))
    }

    pub fn on_enter(s: T) -> RunCriteriaDescriptor {
        (|state: Res<State<T>>, pred: Local<Option<T>>| {
            state
                .transition
                .as_ref()
                .map_or(false, |transition| match transition {
                    StateTransition::Entering(_, entering) => entering == pred.as_ref().unwrap(),
                    StateTransition::Startup => {
                        state.stack.last().unwrap() == pred.as_ref().unwrap()
                    }
                    _ => false,
                })
        })
        .config(|(_, pred)| *pred = Some(Some(s.clone())))
        .chain(should_run_adapter::<T>)
        .after(DriverLabel::of::<T>())
        .label_discard_if_duplicate(StateCallback::Enter.into_label(s))
    }

    pub fn on_exit(s: T) -> RunCriteriaDescriptor {
        (|state: Res<State<T>>, pred: Local<Option<T>>| {
            state
                .transition
                .as_ref()
                .map_or(false, |transition| match transition {
                    StateTransition::ExitingToResume(exiting, _)
                    | StateTransition::ExitingFull(exiting, _) => exiting == pred.as_ref().unwrap(),
                    _ => false,
                })
        })
        .config(|(_, pred)| *pred = Some(Some(s.clone())))
        .chain(should_run_adapter::<T>)
        .after(DriverLabel::of::<T>())
        .label_discard_if_duplicate(StateCallback::Exit.into_label(s))
    }

    pub fn on_pause(s: T) -> RunCriteriaDescriptor {
        (|state: Res<State<T>>, pred: Local<Option<T>>| {
            state
                .transition
                .as_ref()
                .map_or(false, |transition| match transition {
                    StateTransition::Pausing(pausing, _) => pausing == pred.as_ref().unwrap(),
                    _ => false,
                })
        })
        .config(|(_, pred)| *pred = Some(Some(s.clone())))
        .chain(should_run_adapter::<T>)
        .after(DriverLabel::of::<T>())
        .label_discard_if_duplicate(StateCallback::Pause.into_label(s))
    }

    pub fn on_resume(s: T) -> RunCriteriaDescriptor {
        (|state: Res<State<T>>, pred: Local<Option<T>>| {
            state
                .transition
                .as_ref()
                .map_or(false, |transition| match transition {
                    StateTransition::Resuming(_, resuming) => resuming == pred.as_ref().unwrap(),
                    _ => false,
                })
        })
        .config(|(_, pred)| *pred = Some(Some(s.clone())))
        .chain(should_run_adapter::<T>)
        .after(DriverLabel::of::<T>())
        .label_discard_if_duplicate(StateCallback::Resume.into_label(s))
    }

    pub fn on_update_set(s: T) -> SystemSet {
        SystemSet::new().with_run_criteria(Self::on_update(s))
    }

    pub fn on_inactive_update_set(s: T) -> SystemSet {
        SystemSet::new().with_run_criteria(Self::on_inactive_update(s))
    }

    pub fn on_enter_set(s: T) -> SystemSet {
        SystemSet::new().with_run_criteria(Self::on_enter(s))
    }

    pub fn on_exit_set(s: T) -> SystemSet {
        SystemSet::new().with_run_criteria(Self::on_exit(s))
    }

    pub fn on_pause_set(s: T) -> SystemSet {
        SystemSet::new().with_run_criteria(Self::on_pause(s))
    }

    pub fn on_resume_set(s: T) -> SystemSet {
        SystemSet::new().with_run_criteria(Self::on_resume(s))
    }

    /// Creates a driver set for the State.
    ///
    /// Important note: this set must be inserted **before** all other state-dependant sets to work
    /// properly!
    pub fn get_driver() -> SystemSet {
        SystemSet::default().with_run_criteria(state_cleaner::<T>.label(DriverLabel::of::<T>()))
    }

    pub fn new(initial: T) -> Self {
        Self {
            stack: vec![initial],
            transition: Some(StateTransition::PreStartup),
            scheduled: None,
            end_next_loop: false,
        }
    }

    /// Schedule a state change that replaces the active state with the given state.
    /// This will fail if there is a scheduled operation, or if the given `state` matches the
    /// current state
    pub fn set(&mut self, state: T) -> Result<(), StateError> {
        if self.stack.last().unwrap() == &state {
            return Err(StateError::AlreadyInState);
        }

        if self.scheduled.is_some() {
            return Err(StateError::StateAlreadyQueued);
        }

        self.scheduled = Some(ScheduledOperation::Set(state));
        Ok(())
    }

    /// Same as [`Self::set`], but if there is already a next state, it will be overwritten
    /// instead of failing
    pub fn overwrite_set(&mut self, state: T) -> Result<(), StateError> {
        if self.stack.last().unwrap() == &state {
            return Err(StateError::AlreadyInState);
        }

        self.scheduled = Some(ScheduledOperation::Set(state));
        Ok(())
    }

    /// Schedule a state change that replaces the full stack with the given state.
    /// This will fail if there is a scheduled operation, or if the given `state` matches the
    /// current state
    pub fn replace(&mut self, state: T) -> Result<(), StateError> {
        if self.stack.last().unwrap() == &state {
            return Err(StateError::AlreadyInState);
        }

        if self.scheduled.is_some() {
            return Err(StateError::StateAlreadyQueued);
        }

        self.scheduled = Some(ScheduledOperation::Replace(state));
        Ok(())
    }

    /// Same as [`Self::replace`], but if there is already a next state, it will be overwritten
    /// instead of failing
    pub fn overwrite_replace(&mut self, state: T) -> Result<(), StateError> {
        if self.stack.last().unwrap() == &state {
            return Err(StateError::AlreadyInState);
        }

        self.scheduled = Some(ScheduledOperation::Replace(state));
        Ok(())
    }

    /// Same as [`Self::set`], but does a push operation instead of a next operation
    pub fn push(&mut self, state: T) -> Result<(), StateError> {
        if self.stack.last().unwrap() == &state {
            return Err(StateError::AlreadyInState);
        }

        if self.scheduled.is_some() {
            return Err(StateError::StateAlreadyQueued);
        }

        self.scheduled = Some(ScheduledOperation::Push(state));
        Ok(())
    }

    /// Same as [`Self::push`], but if there is already a next state, it will be overwritten
    /// instead of failing
    pub fn overwrite_push(&mut self, state: T) -> Result<(), StateError> {
        if self.stack.last().unwrap() == &state {
            return Err(StateError::AlreadyInState);
        }

        self.scheduled = Some(ScheduledOperation::Push(state));
        Ok(())
    }

    /// Same as [`Self::set`], but does a pop operation instead of a set operation
    pub fn pop(&mut self) -> Result<(), StateError> {
        if self.scheduled.is_some() {
            return Err(StateError::StateAlreadyQueued);
        }

        if self.stack.len() == 1 {
            return Err(StateError::StackEmpty);
        }

        self.scheduled = Some(ScheduledOperation::Pop);
        Ok(())
    }

    /// Same as [`Self::pop`], but if there is already a next state, it will be overwritten
    /// instead of failing
    pub fn overwrite_pop(&mut self) -> Result<(), StateError> {
        if self.stack.len() == 1 {
            return Err(StateError::StackEmpty);
        }
        self.scheduled = Some(ScheduledOperation::Pop);
        Ok(())
    }

    pub fn current(&self) -> &T {
        self.stack.last().unwrap()
    }

    pub fn inactives(&self) -> &[T] {
        self.stack.split_last().map(|(_, rest)| rest).unwrap()
    }
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Attempted to change the state to the current state.")]
    AlreadyInState,
    #[error("Attempted to queue a state change, but there was already a state queued.")]
    StateAlreadyQueued,
    #[error("Attempted to queue a pop, but there is nothing to pop.")]
    StackEmpty,
}

fn should_run_adapter<T: StateData>(In(cmp_result): In<bool>, state: Res<State<T>>) -> ShouldRun {
    if state.end_next_loop {
        return ShouldRun::No;
    }
    if cmp_result {
        ShouldRun::YesAndCheckAgain
    } else {
        ShouldRun::NoAndCheckAgain
    }
}

fn state_cleaner<T: StateData>(
    mut state: ResMut<State<T>>,
    mut prep_exit: Local<bool>,
) -> ShouldRun {
    if *prep_exit {
        *prep_exit = false;
        if state.scheduled.is_none() {
            state.end_next_loop = true;
            return ShouldRun::YesAndCheckAgain;
        }
    } else if state.end_next_loop {
        state.end_next_loop = false;
        return ShouldRun::No;
    }
    match state.scheduled.take() {
        Some(ScheduledOperation::Set(next)) => {
            state.transition = Some(StateTransition::ExitingFull(
                state.stack.last().unwrap().clone(),
                next,
            ));
        }
        Some(ScheduledOperation::Replace(next)) => {
            if state.stack.len() <= 1 {
                state.transition = Some(StateTransition::ExitingFull(
                    state.stack.last().unwrap().clone(),
                    next,
                ));
            } else {
                state.scheduled = Some(ScheduledOperation::Replace(next));
                match state.transition.take() {
                    Some(StateTransition::ExitingToResume(p, n)) => {
                        state.stack.pop();
                        state.transition = Some(StateTransition::Resuming(p, n));
                    }
                    _ => {
                        state.transition = Some(StateTransition::ExitingToResume(
                            state.stack[state.stack.len() - 1].clone(),
                            state.stack[state.stack.len() - 2].clone(),
                        ));
                    }
                }
            }
        }
        Some(ScheduledOperation::Push(next)) => {
            let last_type_id = state.stack.last().unwrap().clone();
            state.transition = Some(StateTransition::Pausing(last_type_id, next));
        }
        Some(ScheduledOperation::Pop) => {
            state.transition = Some(StateTransition::ExitingToResume(
                state.stack[state.stack.len() - 1].clone(),
                state.stack[state.stack.len() - 2].clone(),
            ));
        }
        None => match state.transition.take() {
            Some(StateTransition::ExitingFull(p, n)) => {
                state.transition = Some(StateTransition::Entering(p, n.clone()));
                *state.stack.last_mut().unwrap() = n;
            }
            Some(StateTransition::Pausing(p, n)) => {
                state.transition = Some(StateTransition::Entering(p, n.clone()));
                state.stack.push(n);
            }
            Some(StateTransition::ExitingToResume(p, n)) => {
                state.stack.pop();
                state.transition = Some(StateTransition::Resuming(p, n));
            }
            Some(StateTransition::PreStartup) => {
                state.transition = Some(StateTransition::Startup);
            }
            _ => {}
        },
    };
    if state.transition.is_none() {
        *prep_exit = true;
    }

    ShouldRun::YesAndCheckAgain
}