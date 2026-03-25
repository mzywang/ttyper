pub mod accuracy_data;
pub mod cli;
pub mod fraction;
pub mod results;
pub mod test;
pub mod test_event;
pub mod test_word;
pub mod timing_data;

pub use accuracy_data::AccuracyData;
pub use cli::{Command, Opt};
pub use fraction::Fraction;
pub use results::Results;
pub use test::Test;
pub use test_event::TestEvent;
pub use test_word::TestWord;
pub use timing_data::TimingData;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    Test,
    Results,
}
