use smlang::statemachine;

statemachine! {
    transitions: {
        *Init + Start = Idle,
        Idle | Recovery + WantsCollection = Collection,
        Idle + NoConfig = Calibration,
        Collection + WantsProcessing = Processing,
        Calibration + Configured = Idle,
        Fault + FaultCleared = Idle,
        _ + FaultDetected = Fault,
    }
}

pub async fn calibrate() {

}

pub async fn collect() {

}

pub async fn fault() {

}

pub async fn idle() {

}

pub async fn init() {

}

pub async fn process() {

}

pub async fn recover() {

}

#[cfg(test)]
mod tests {
    use argus::state_machine::*;
    use argus::traits;

    #[test]
    fn sm_should_transition_between_states_on_event() {
        let mut sm = StateMachine::new(traits::Context {});
    
        // Should be in init by default
        assert!(*sm.state() == States::Init);
    
        // Should transition to idle on start
        _ = sm.process_event(Events::Start).unwrap();
        assert!(*sm.state() == States::Idle);
    
        // Should transition to calibration when there is no config
        _ = sm.process_event(Events::NoConfig).unwrap();
        assert!(*sm.state() == States::Calibration);
    
        // Should transition back to idle when configured
        _ = sm.process_event(Events::Configured).unwrap();
        assert!(*sm.state() == States::Idle);
    
        // Should transition to collection when asked
        _ = sm.process_event(Events::WantsCollection).unwrap();
        assert!(*sm.state() == States::Collection);
    
        // Should transition to processing when asked
        _ = sm.process_event(Events::WantsProcessing).unwrap();
        assert!(*sm.state() == States::Processing);
    }
}
