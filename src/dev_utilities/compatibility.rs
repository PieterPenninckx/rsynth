pub trait NotInApplication {}
pub trait NotInCrateRsynth {}

macro_rules! not_in_application {
    () => {
        NotInApplication
    }
}

macro_rules! not_in_crate_rsynth {
    () => {
        NotInCrateRsynth
    }
}

macro_rules! all_traits {
    () => {
        [not_in_application!(), not_in_crate_rsynth!()]
    }
}

