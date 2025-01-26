use crosscut_protocol::command::SerializedCommandToRuntime;
use tokio::sync::mpsc;

pub type CommandsToRuntimeRx =
    mpsc::UnboundedReceiver<SerializedCommandToRuntime>;
pub type CommandsToRuntimeTx =
    mpsc::UnboundedSender<SerializedCommandToRuntime>;
