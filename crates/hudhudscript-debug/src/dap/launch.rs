use super::*;

impl DapServer {
    pub(super) fn handle_launch<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        if let Some(args) = &request.arguments {
            if let Ok(launch_args) = serde_json::from_value::<LaunchArguments>(args.clone()) {
                self.program = launch_args.program;
                self.stop_on_entry = launch_args.stop_on_entry.unwrap_or(false);
            }
        }

        self.launched = true;

        let resp = self.make_response(request, true, None);
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_set_breakpoints<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        let args: SetBreakpointsArguments = match &request.arguments {
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            None => {
                let resp = self.make_error_response(request, "missing arguments");
                return Self::write_message(writer, &resp);
            }
        };

        let source_path = args
            .source
            .path
            .clone()
            .unwrap_or_else(|| args.source.name.clone().unwrap_or_default());

        // Remove old breakpoints for this source.
        if let Some(old_ids) = self.source_breakpoints.remove(&source_path) {
            for id in old_ids {
                self.debugger.remove_breakpoint(id);
            }
        }

        // Set new breakpoints.
        let source_bps = args.breakpoints.unwrap_or_default();
        let mut bp_ids = Vec::with_capacity(source_bps.len());
        let mut result_bps = Vec::with_capacity(source_bps.len());

        for sbp in &source_bps {
            let id = if let Some(ref cond) = sbp.condition {
                self.debugger.add_conditional_breakpoint(
                    source_path.clone(),
                    sbp.line,
                    cond.clone(),
                )
            } else {
                self.debugger.add_breakpoint(source_path.clone(), sbp.line)
            };
            bp_ids.push(id);
            result_bps.push(serde_json::json!({
                "id": id,
                "verified": true,
                "line": sbp.line,
                "source": {
                    "path": source_path,
                },
            }));
        }

        self.source_breakpoints.insert(source_path, bp_ids);

        let resp = self.make_response(
            request,
            true,
            Some(serde_json::json!({ "breakpoints": result_bps })),
        );
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_configuration_done<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        let resp = self.make_response(request, true, None);
        Self::write_message(writer, &resp)?;

        // If stopOnEntry was requested, pause immediately.
        if self.stop_on_entry {
            self.debugger.pause();
            let event = self.make_event(
                "stopped",
                Some(serde_json::json!({
                    "reason": "entry",
                    "threadId": THREAD_ID,
                    "allThreadsStopped": true,
                })),
            );
            Self::write_message(writer, &event)?;
        }

        Ok(())
    }

    pub(super) fn handle_threads<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        let resp = self.make_response(
            request,
            true,
            Some(serde_json::json!({
                "threads": [{
                    "id": THREAD_ID,
                    "name": THREAD_NAME,
                }]
            })),
        );
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_stack_trace<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        let call_frames = self.debugger.call_frames();
        let mut frames = Vec::new();

        // Build stack frames from the debugger's call stack.
        // The top of the stack (most recent call) is at the end of the vec,
        // but DAP expects the most recent frame first.
        for (i, frame) in call_frames.iter().rev().enumerate() {
            let source = match &frame.file {
                Some(f) => serde_json::json!({ "path": f, "name": f }),
                None => self.current_source_json(),
            };
            let line = frame
                .line
                .or_else(|| self.debugger.current_location().map(|(_, l)| l))
                .unwrap_or(0);
            frames.push(serde_json::json!({
                "id": i as i64,
                "name": frame.name,
                "source": source,
                "line": line,
                "column": 1,
            }));
        }

        // If the call stack is empty, still report the current location as
        // the top-level frame (global scope).
        if frames.is_empty() {
            if let Some((file, line)) = self.debugger.current_location() {
                frames.push(serde_json::json!({
                    "id": 0,
                    "name": "<global>",
                    "source": {
                        "path": file,
                        "name": file,
                    },
                    "line": line,
                    "column": 1,
                }));
            }
        }

        let total = frames.len() as i64;
        let resp = self.make_response(
            request,
            true,
            Some(serde_json::json!({
                "stackFrames": frames,
                "totalFrames": total,
            })),
        );
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_scopes<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        // We report a single "Local" scope with variablesReference = 1.
        let scopes = vec![serde_json::json!({
            "name": "Local",
            "variablesReference": 1,
            "expensive": false,
        })];

        let resp = self.make_response(request, true, Some(serde_json::json!({ "scopes": scopes })));
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_variables<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        let args: VariablesArguments = match &request.arguments {
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            None => {
                let resp = self.make_error_response(request, "missing arguments");
                return Self::write_message(writer, &resp);
            }
        };

        let variables = self
            .variable_store
            .get(&args.variables_reference)
            .cloned()
            .unwrap_or_default();

        let resp = self.make_response(
            request,
            true,
            Some(serde_json::json!({ "variables": variables })),
        );
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_continue<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        self.debugger.resume();
        let resp = self.make_response(
            request,
            true,
            Some(serde_json::json!({ "allThreadsContinued": true })),
        );
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_next<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        self.debugger.step(StepMode::Over);
        let resp = self.make_response(request, true, None);
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_step_in<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        self.debugger.step(StepMode::Into);
        let resp = self.make_response(request, true, None);
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_step_out<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        self.debugger.step(StepMode::Out);
        let resp = self.make_response(request, true, None);
        Self::write_message(writer, &resp)
    }

    pub(super) fn handle_disconnect<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        self.disconnected = true;
        let resp = self.make_response(request, true, None);
        Self::write_message(writer, &resp)
    }
}
