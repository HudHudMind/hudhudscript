use super::ContinuationResume;
use hudhudscript_bytecode::{gc, DynamicObject, Value16};

pub(crate) struct GovernanceDispatchState {
    pub(crate) dst: u8,
    pub(crate) response: hudhudscript_bytecode::ObjMap,
}

impl GovernanceDispatchState {
    pub(super) fn finish(self, value: Value16) -> ContinuationResume {
        let mut response = self.response;
        if let Some(object) = value.as_object() {
            for (key, value) in object.iter() {
                response.insert(key.clone(), *value);
            }
        }
        response.insert("dispatched".to_string(), Value16::bool_(true));
        ContinuationResume::Complete {
            dst: self.dst,
            value: Value16::object(response),
        }
    }

    pub(super) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        for (_, value) in self.response.iter() {
            gc::trace_value(*value, gray);
        }
    }
}
