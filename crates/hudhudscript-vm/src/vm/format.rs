use super::OutputLocale;
use crate::vm::VM;
use hudhudscript_bytecode::{PromiseState16, Value16};

impl VM {
    pub fn value_to_string<V: Into<Value16>>(&self, value: V) -> String {
        let v = value.into();
        self.value_to_string_inner(&v)
    }

    pub(crate) fn value_to_string_inner(&self, value: &Value16) -> String {
        if value.is_null() {
            "null".to_string()
        } else if let Some(b) = value.as_bool() {
            b.to_string()
        } else if let Some(i) = value.as_int() {
            let formatted = i.to_string();
            if self.locale == OutputLocale::Arabic {
                self.to_arabic_numerals(&formatted)
            } else {
                formatted
            }
        } else if let Some(n) = value.as_number() {
            let formatted = if n.fract() == 0.0 && n.is_finite() {
                format!("{}", n as i64)
            } else {
                n.to_string()
            };
            if self.locale == OutputLocale::Arabic {
                self.to_arabic_numerals(&formatted)
            } else {
                formatted
            }
        } else if let Some(s) = value.as_string() {
            s
        } else if let Some(arr) = value.as_array() {
            let elements: Vec<String> = arr.iter().map(|v| self.value_to_string_inner(v)).collect();
            format!("[{}]", elements.join(", "))
        } else if let Some(obj) = value.as_object() {
            let mut pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, self.value_to_string_inner(v)))
                .collect();
            pairs.sort();
            format!("{{{}}}", pairs.join(", "))
        } else if let Some(func) = value.as_function_data() {
            format!("<function {}({})>", func.name, func.params.join(", "))
        } else if let Some(opt) = value.as_option() {
            match opt {
                Some(v) => format!("Some({})", self.value_to_string_inner(v)),
                None => "None".to_string(),
            }
        } else if let Some(r) = value.as_result() {
            match r {
                Ok(v) => format!("Ok({})", self.value_to_string_inner(v)),
                Err(e) => format!("Err({})", e),
            }
        } else if let Some(ps) = value.as_promise_state() {
            match ps {
                PromiseState16::Resolved(v) => {
                    format!("Promise(resolved: {})", self.value_to_string_inner(v))
                }
                PromiseState16::Pending => "Promise(pending)".to_string(),
                PromiseState16::Rejected(msg) => format!("Promise(rejected: {})", msg),
                PromiseState16::AsyncPending(id) => {
                    format!("<promise: async-pending({})>", id)
                }
            }
        } else if let Some(c) = value.as_class_data() {
            format!("<class {}>", c.name)
        } else if let Some(inst) = value.as_instance_data() {
            let f: Vec<String> = inst
                .fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, self.value_to_string_inner(v)))
                .collect();
            format!("{} {{{}}}", inst.class_name, f.join(", "))
        } else if let Some(d) = value.as_data_data() {
            let f: Vec<String> = d
                .fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, self.value_to_string_inner(v)))
                .collect();
            if d.type_name.is_empty() {
                format!("data {{{}}}", f.join(", "))
            } else {
                format!("{} {{{}}}", d.type_name, f.join(", "))
            }
        } else if let Some(t) = value.as_tool_ref() {
            format!("<tool {}:{}>", t.server, t.tool_name)
        } else if let Some(r) = value.as_resource_ref() {
            format!("<resource {}:{}>", r.server, r.uri)
        } else if let Some(items) = value.as_set() {
            let elements: Vec<String> = items
                .iter()
                .map(|v| self.value_to_string_inner(v))
                .collect();
            format!("Set{{{}}}", elements.join(", "))
        } else if let Some(pairs) = value.as_map_pairs() {
            let elements: Vec<String> = pairs
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{} => {}",
                        self.value_to_string_inner(k),
                        self.value_to_string_inner(v)
                    )
                })
                .collect();
            format!("Map{{{}}}", elements.join(", "))
        } else if let Some(state) = value.as_generator_state() {
            let st = state.lock();
            format!(
                "<generator yielded={} {}>",
                st.buffered.len(),
                if st.is_done() { "done" } else { "active" }
            )
        } else {
            format!("{:?}", value)
        }
    }

    /// Convert ASCII digits to Arabic-Indic numerals
    pub(crate) fn to_arabic_numerals(&self, s: &str) -> String {
        s.chars()
            .map(|ch| match ch {
                '0' => '٠',
                '1' => '١',
                '2' => '٢',
                '3' => '٣',
                '4' => '٤',
                '5' => '٥',
                '6' => '٦',
                '7' => '٧',
                '8' => '٨',
                '9' => '٩',
                _ => ch,
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Built-in functions
    // -----------------------------------------------------------------------
}
