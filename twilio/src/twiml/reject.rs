use super::{format_xml_string, Action};

#[derive(Debug, Default)]
pub struct Reject {
    pub reason: RejectReason,
}

impl Action for Reject {
    fn as_twiml(&self) -> String {
        let reason = match self.reason {
            RejectReason::Rejected => "rejected",
            RejectReason::Busy => "busy",
        };

        format_xml_string("Reject", &[("reason", reason)], "")
    }
}

#[derive(Debug, Default)]
pub enum RejectReason {
    #[default]
    Rejected,
    Busy,
}
