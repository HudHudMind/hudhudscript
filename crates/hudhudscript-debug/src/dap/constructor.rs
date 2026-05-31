use super::*;

impl DapServer {
    /// Create a new DAP server wrapping a fresh [`Debugger`].
    pub fn new() -> Self {
        Self {
            debugger: Debugger::new(),
            seq: 1,
            initialized: false,
            launched: false,
            disconnected: false,
            program: None,
            stop_on_entry: false,
            source_breakpoints: HashMap::new(),
            variable_store: HashMap::new(),
        }
    }

    /// Create a DAP server wrapping an existing [`Debugger`].
    pub fn with_debugger(debugger: Debugger) -> Self {
        Self {
            debugger,
            seq: 1,
            initialized: false,
            launched: false,
            disconnected: false,
            program: None,
            stop_on_entry: false,
            source_breakpoints: HashMap::new(),
            variable_store: HashMap::new(),
        }
    }

    /// Returns a reference to the inner debugger.
    pub fn debugger(&self) -> &Debugger {
        &self.debugger
    }

    /// Returns a mutable reference to the inner debugger.
    pub fn debugger_mut(&mut self) -> &mut Debugger {
        &mut self.debugger
    }

    /// Whether the server has received a disconnect request.
    pub fn is_disconnected(&self) -> bool {
        self.disconnected
    }

    // ---------------------------------------------------------------------
    // Wire protocol: reading
    // ---------------------------------------------------------------------

    /// Read a single DAP message from a buffered reader.
    ///
    /// Returns `None` on EOF.
    pub fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<DapRequest>> {
        // Read headers until we find Content-Length.
        let mut content_length: Option<usize> = None;
        loop {
            let mut header_line = String::new();
            let bytes_read = reader.read_line(&mut header_line)?;
            if bytes_read == 0 {
                return Ok(None); // EOF
            }
            let trimmed = header_line.trim();
            if trimmed.is_empty() {
                // End of headers.
                break;
            }
            if let Some(val) = trimmed.strip_prefix("Content-Length:") {
                content_length = Some(
                    val.trim()
                        .parse::<usize>()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
                );
            }
            // Ignore other headers (Content-Type, etc.).
        }

        let length = content_length
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length"))?;

        let mut body = vec![0u8; length];
        reader.read_exact(&mut body)?;

        let request: DapRequest = serde_json::from_slice(&body)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Some(request))
    }

    // ---------------------------------------------------------------------
    // Wire protocol: writing
    // ---------------------------------------------------------------------

    /// Write a DAP message (response or event) to the writer.
    pub(super) fn write_message<W: Write>(writer: &mut W, msg: &DapMessage) -> io::Result<()> {
        let body = serde_json::to_string(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        writer.flush()
    }

    pub(super) fn next_seq(&mut self) -> i64 {
        let s = self.seq;
        self.seq += 1;
        s
    }

    pub(super) fn make_response(
        &mut self,
        request: &DapRequest,
        success: bool,
        body: Option<Value>,
    ) -> DapMessage {
        DapMessage::Response(DapResponse {
            seq: self.next_seq(),
            request_seq: request.seq,
            success,
            command: request.command.clone(),
            message: None,
            body,
        })
    }

    pub(super) fn make_error_response(
        &mut self,
        request: &DapRequest,
        message: &str,
    ) -> DapMessage {
        DapMessage::Response(DapResponse {
            seq: self.next_seq(),
            request_seq: request.seq,
            success: false,
            command: request.command.clone(),
            message: Some(message.to_string()),
            body: None,
        })
    }

    pub(super) fn make_event(&mut self, event: &str, body: Option<Value>) -> DapMessage {
        DapMessage::Event(DapEvent {
            seq: self.next_seq(),
            event: event.to_string(),
            body,
        })
    }

    // ---------------------------------------------------------------------
    // Public: process a single request and write response(s)
    // ---------------------------------------------------------------------

    /// Handle a single DAP request and write the response (and any events)
    /// to `writer`.
    pub fn handle_request<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        match request.command.as_str() {
            "initialize" => self.handle_initialize(request, writer),
            "launch" => self.handle_launch(request, writer),
            "setBreakpoints" => self.handle_set_breakpoints(request, writer),
            "configurationDone" => self.handle_configuration_done(request, writer),
            "threads" => self.handle_threads(request, writer),
            "stackTrace" => self.handle_stack_trace(request, writer),
            "scopes" => self.handle_scopes(request, writer),
            "variables" => self.handle_variables(request, writer),
            "continue" => self.handle_continue(request, writer),
            "next" => self.handle_next(request, writer),
            "stepIn" => self.handle_step_in(request, writer),
            "stepOut" => self.handle_step_out(request, writer),
            "disconnect" => self.handle_disconnect(request, writer),
            "evaluate" => self.handle_evaluate(request, writer),
            "setExceptionBreakpoints" => self.handle_set_exception_breakpoints(request, writer),
            _ => {
                let resp = self.make_error_response(
                    request,
                    &format!("unsupported command: {}", request.command),
                );
                Self::write_message(writer, &resp)
            }
        }
    }

    /// Run the server main loop over `reader`/`writer` streams, processing
    /// requests until disconnect or EOF.
    pub fn run<R: Read, W: Write>(&mut self, reader: R, writer: &mut W) -> io::Result<()> {
        let mut buf_reader = BufReader::new(reader);
        while let Some(request) = Self::read_message(&mut buf_reader)? {
            self.handle_request(&request, writer)?;
            if self.disconnected {
                break;
            }
        }
        Ok(())
    }

    /// Run the DAP server on stdin/stdout.
    pub fn run_stdio(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        self.run(stdin.lock(), &mut stdout)
    }

    // ---------------------------------------------------------------------
    // Notification helpers (called by the runtime when execution pauses)
    // ---------------------------------------------------------------------

    /// Send a `stopped` event to the client. This should be called by the
    /// runtime when `Debugger::on_statement` returns `true`.
    pub fn send_stopped_event<W: Write>(
        &mut self,
        writer: &mut W,
        variables: Vec<Variable>,
    ) -> io::Result<()> {
        let reason = match self.debugger.pause_reason() {
            Some(PauseReason::Breakpoint(_)) => "breakpoint",
            Some(PauseReason::Step) => "step",
            Some(PauseReason::Explicit) => "pause",
            Some(PauseReason::Exception(_)) => "exception",
            None => "unknown",
        };

        // Store variables for later `scopes` / `variables` requests.
        self.variable_store.clear();
        if !variables.is_empty() {
            self.variable_store.insert(1, variables);
        } else {
            // Fall back to the debugger's own scope variables if no external
            // variables were passed. This ensures `variables` requests work
            // even when the runtime only updates scope via set_scope_variables.
            self.populate_variables_from_scope();
        }

        let event = self.make_event(
            "stopped",
            Some(serde_json::json!({
                "reason": reason,
                "threadId": THREAD_ID,
                "allThreadsStopped": true,
            })),
        );
        Self::write_message(writer, &event)
    }

    /// Send a `terminated` event to indicate the debuggee has ended.
    pub fn send_terminated_event<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let event = self.make_event("terminated", None);
        Self::write_message(writer, &event)
    }

    /// Send an `output` event (console output from the debuggee).
    pub fn send_output_event<W: Write>(
        &mut self,
        writer: &mut W,
        category: &str,
        output: &str,
    ) -> io::Result<()> {
        let event = self.make_event(
            "output",
            Some(serde_json::json!({
                "category": category,
                "output": output,
            })),
        );
        Self::write_message(writer, &event)
    }

    // ---------------------------------------------------------------------
    // Request handlers
    // ---------------------------------------------------------------------

    pub(super) fn handle_initialize<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        self.initialized = true;

        // Report capabilities.
        let capabilities = serde_json::json!({
            "supportsConfigurationDoneRequest": true,
            "supportsFunctionBreakpoints": false,
            "supportsConditionalBreakpoints": true,
            "supportsLogPoints": true,
            "supportsExceptionInfoRequest": true,
            "exceptionBreakpointFilters": [
                {
                    "filter": "all",
                    "label": "All Exceptions",
                    "default": false,
                    "supportsCondition": false,
                },
                {
                    "filter": "uncaught",
                    "label": "Uncaught Exceptions",
                    "default": true,
                    "supportsCondition": false,
                }
            ],
            "supportsEvaluateForHovers": true,
            "supportsStepBack": false,
            "supportsSetVariable": false,
            "supportsRestartFrame": false,
            "supportsStepInTargetsRequest": false,
            "supportTerminateDebuggee": true,
            "supportsDelayedStackTraceLoading": false,
        });

        let resp = self.make_response(request, true, Some(capabilities));
        Self::write_message(writer, &resp)?;

        // Send the `initialized` event.
        let event = self.make_event("initialized", None);
        Self::write_message(writer, &event)
    }
}
