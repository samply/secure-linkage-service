#[cfg(test)]

mod tests {
    // use secure_linkage_service::entities::*;
    use secure_linkage_service::transformations::*;

    #[test]
    fn initial_state_should() {
        let state = create_initial_state();
        assert_eq!(state.epilinkers.len(), 0);
        assert_eq!(state.computations.len(), 0);
    }
}
