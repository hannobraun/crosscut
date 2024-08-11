use std::fmt::Debug;

pub trait Host {
    type Effect: HostEffect;

    fn function(name: &str) -> Option<Self::Effect>;
}

pub trait HostEffect:
    Clone + Debug + Eq + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    fn to_number(self) -> u8;
}

impl HostEffect for () {
    fn to_number(self) -> u8 {
        0
    }
}

pub struct NoHost {}

impl Host for NoHost {
    type Effect = ();

    fn function(_name: &str) -> Option<Self::Effect> {
        None
    }
}
