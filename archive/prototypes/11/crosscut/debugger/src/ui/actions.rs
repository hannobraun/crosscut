use tokio::sync::mpsc;

use crate::model::UserAction;

pub type ActionsTx = mpsc::UnboundedSender<UserAction>;

pub async fn send_action(action: UserAction, actions: ActionsTx) {
    if let Err(err) = actions.send(action) {
        log::error!(
            "Sending a UI action failed, as the receive is no longer \
            available: {err:#?}\n\
            \n\
            This is most likely a bug in the Crosscut debugger."
        );
    }
}
