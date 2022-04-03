use crate::provider::*;
use alloc::vec::Vec;
use nrq_engine::{structs::OtherClientInfo, RQError};

impl<P> crate::Client<P>
where
    P: ClientProvider,
{
    pub(crate) async fn process_push_param(
        &self,
        other_clients: Vec<OtherClientInfo>,
    ) -> Result<(), RQError> {
        tracing::debug!(target = "rs_qq", "{:?}", other_clients);
        // TODO merge part and dispatch to handler
        Ok(())
    }
}
