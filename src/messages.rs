use crate::types::Results;

#[derive(Debug, PartialEq, Clone)]
pub enum Msg {
    AppClose,
    ShowResults(Results),
    RestartTest,
    StartTest(Vec<String>),
    None,
}
