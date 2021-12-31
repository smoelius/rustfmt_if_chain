fn main() -> PhilosophicalResult<()> {
    if_chain! {
        if let Some(it) = tree.falls_in(forest);
        if !listeners.any(|one| one.around_to_hear(it));
        if !it.makes_a_sound();
        then {
            return Err(PhilosophicalError::new());
        }
    }
    Ok(())
}
