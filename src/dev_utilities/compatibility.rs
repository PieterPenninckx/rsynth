pub trait NotInApplication {}
pub trait NotInCrateRsynth {}

macro_rules! traits_for_rsynth {
    () => {
        (NotInApplication,)
    }
}

macro_rules! traits_for_application {
    () => {
        (NotInCrateRsynth,)
    }
}
