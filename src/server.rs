//! The BWOC MCP server: a `rmcp` tool router over the Hybrid [`Bridge`].
//!
//! Every BWOC verb is wired as an MCP tool. Tiering is enforced at call time
//! against the [`Posture`] (read tools always on; write/exec/dangerous gated by
//! the `--allow-*` flags) rather than at registration, so a client always sees
//! the full catalog and gets a clear refusal for a disabled tier.
//!
//! The full catalog is reached via the shell-out half of the bridge
//! (`bwoc <verb> --json`, or `text()` for the handful of verbs without a
//! `--json` twin). The in-process `bridge::core` path is a performance seam for
//! team/task/inbox; see `docs/PLAN.md` §Phase 3.

use crate::bridge::Bridge;
use crate::cli::Posture;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone)]
pub struct BwocMcp {
    bridge: Bridge,
    posture: Posture,
    // Read by the `#[tool_handler]`-generated dispatch; the analyzer can't see
    // through the macro, hence the allow.
    #[allow(dead_code)]
    tool_router: ToolRouter<BwocMcp>,
}

// ---- tool argument schemas -------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentArg {
    /// Agent path or name, e.g. `agents/agent-yudi` or `yudi`.
    pub agent: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NameArg {
    /// Agent name or id, e.g. `yudi` or `agent-yudi`.
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InboxArgs {
    /// Recipient agent name or id.
    pub agent: String,
    /// Optional: show only the last N messages.
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TeamArg {
    /// Team id.
    pub team: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SendArgs {
    /// Recipient agent name or id.
    pub agent: String,
    /// Message body to append to the agent's inbox.
    pub message: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunArgs {
    /// Agent name or id to run the task as.
    pub agent: String,
    /// Task prompt to execute headlessly.
    pub task: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskAddArgs {
    /// Team the task belongs to.
    pub team: String,
    /// Human-readable task title.
    pub title: String,
    /// Optional comma-separated task ids that gate this one.
    #[serde(default)]
    pub deps: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskClaimArgs {
    /// Team id.
    pub team: String,
    /// Task id to claim/complete.
    pub task: String,
    /// Claiming/completing agent id (must be a team member / the claimant).
    pub agent: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TeamCreateArgs {
    /// Team id to create.
    pub team: String,
    /// Comma-separated member agent ids.
    pub members: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemoryShowArgs {
    /// Memory entry name.
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemoryPutArgs {
    /// Memory entry name.
    pub name: String,
    /// Entry content.
    pub content: String,
}

// ---- tools -----------------------------------------------------------------

#[tool_router]
impl BwocMcp {
    pub fn new(bridge: Bridge, posture: Posture) -> Self {
        Self {
            bridge,
            posture,
            tool_router: Self::tool_router(),
        }
    }

    // ====================== READ (always available) ======================

    /// Liveness check — returns the targeted workspace path. In-process.
    #[tool(description = "Ping the BWOC MCP server; returns the active workspace path.")]
    async fn bwoc_ping(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "pong: {}",
            self.bridge.workspace.display()
        ))]))
    }

    #[tool(description = "List the agents registered in the workspace (id, status, role).")]
    async fn bwoc_list(&self) -> Result<CallToolResult, McpError> {
        self.json_tool(&["list"]).await
    }

    #[tool(description = "Show health and identity for one agent.")]
    async fn bwoc_status(
        &self,
        Parameters(AgentArg { agent }): Parameters<AgentArg>,
    ) -> Result<CallToolResult, McpError> {
        self.json_tool(&["status", &agent]).await
    }

    #[tool(description = "One-card system status: version, release, phase, workspace + agents.")]
    async fn bwoc_info(&self) -> Result<CallToolResult, McpError> {
        self.json_tool(&["info"]).await
    }

    #[tool(description = "Report fleet-wide Aparihāniya-dhamma 7 health signals.")]
    async fn bwoc_fleet(&self) -> Result<CallToolResult, McpError> {
        self.json_tool(&["fleet"]).await
    }

    #[tool(description = "List running agent sessions detected via markers + process scan.")]
    async fn bwoc_sessions(&self) -> Result<CallToolResult, McpError> {
        self.json_tool(&["sessions"]).await
    }

    #[tool(description = "Read an agent's Kalyāṇamitta-7 trust profile (declared + required).")]
    async fn bwoc_trust(
        &self,
        Parameters(AgentArg { agent }): Parameters<AgentArg>,
    ) -> Result<CallToolResult, McpError> {
        self.json_tool(&["trust", &agent]).await
    }

    #[tool(description = "List teams in the workspace with member + task counts.")]
    async fn bwoc_team_list(&self) -> Result<CallToolResult, McpError> {
        // `team list` has no --json twin; return its text layout.
        self.text_tool(&["team", "list"]).await
    }

    #[tool(description = "List a team's shared tasks with state + claimant.")]
    async fn bwoc_task_list(
        &self,
        Parameters(TeamArg { team }): Parameters<TeamArg>,
    ) -> Result<CallToolResult, McpError> {
        self.json_tool(&["task", "list", &team]).await
    }

    #[tool(description = "Read an agent's inbox messages.")]
    async fn bwoc_inbox_read(
        &self,
        Parameters(InboxArgs { agent, limit }): Parameters<InboxArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut args: Vec<String> = vec!["inbox".into(), agent];
        if let Some(n) = limit {
            args.push("--limit".into());
            args.push(n.to_string());
        }
        self.json_owned(args).await
    }

    #[tool(description = "List user-authored workspace memory entries.")]
    async fn bwoc_memory_list(&self) -> Result<CallToolResult, McpError> {
        self.text_tool(&["memory", "list"]).await
    }

    #[tool(description = "Print one workspace memory entry's contents.")]
    async fn bwoc_memory_show(
        &self,
        Parameters(MemoryShowArgs { name }): Parameters<MemoryShowArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.text_tool(&["memory", "show", &name]).await
    }

    #[tool(description = "List peers declared in this workspace's routes.toml.")]
    async fn bwoc_peer_list(&self) -> Result<CallToolResult, McpError> {
        self.text_tool(&["peer", "list"]).await
    }

    // ====================== WRITE (--allow-write) ======================

    #[tool(description = "Send a message to an agent's inbox. Requires --allow-write.")]
    async fn bwoc_send(
        &self,
        Parameters(SendArgs { agent, message }): Parameters<SendArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_write()?;
        self.text_tool(&["send", &agent, &message]).await
    }

    #[tool(description = "Add a task to a team's shared list. Requires --allow-write.")]
    async fn bwoc_task_add(
        &self,
        Parameters(TaskAddArgs { team, title, deps }): Parameters<TaskAddArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_write()?;
        let mut args: Vec<String> = vec!["task".into(), "add".into(), team, title];
        if let Some(d) = deps {
            args.push("--deps".into());
            args.push(d);
        }
        self.json_owned(args).await
    }

    #[tool(description = "Claim a pending, unblocked task as an agent. Requires --allow-write.")]
    async fn bwoc_task_claim(
        &self,
        Parameters(TaskClaimArgs { team, task, agent }): Parameters<TaskClaimArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_write()?;
        self.json_tool(&["task", "claim", "--as", &agent, &team, &task])
            .await
    }

    #[tool(description = "Complete an in-progress task you claimed. Requires --allow-write.")]
    async fn bwoc_task_complete(
        &self,
        Parameters(TaskClaimArgs { team, task, agent }): Parameters<TaskClaimArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_write()?;
        self.json_tool(&["task", "complete", "--as", &agent, &team, &task])
            .await
    }

    #[tool(description = "Create a team with a member list. Requires --allow-write.")]
    async fn bwoc_team_create(
        &self,
        Parameters(TeamCreateArgs { team, members }): Parameters<TeamCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_write()?;
        self.text_tool(&["team", "create", &team, "--members", &members])
            .await
    }

    #[tool(description = "Write a workspace memory entry. Requires --allow-write.")]
    async fn bwoc_memory_put(
        &self,
        Parameters(MemoryPutArgs { name, content }): Parameters<MemoryPutArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_write()?;
        self.text_tool(&["memory", "put", &name, &content]).await
    }

    // ====================== EXEC (--allow-exec) ======================

    #[tool(
        description = "Run a single task non-interactively as an agent and return the result. Requires --allow-exec."
    )]
    async fn bwoc_run(
        &self,
        Parameters(RunArgs { agent, task }): Parameters<RunArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_exec()?;
        // `bwoc run --task <TASK> <AGENT>` — task is a flag, agent positional.
        self.json_tool(&["run", &agent, "--task", &task]).await
    }

    // ====================== DANGEROUS (--allow-dangerous) ======================

    #[tool(description = "Incarnate a new agent from the template. Requires --allow-dangerous.")]
    async fn bwoc_new(
        &self,
        Parameters(NameArg { name }): Parameters<NameArg>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_dangerous()?;
        self.json_tool(&["new", &name]).await
    }

    #[tool(description = "Retire an agent (remove from registry + files). Requires --allow-dangerous.")]
    async fn bwoc_retire(
        &self,
        Parameters(NameArg { name }): Parameters<NameArg>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_dangerous()?;
        // `--yes` is mandatory for scripted/non-TTY destructive ops with --json.
        self.json_tool(&["retire", &name, "--yes"]).await
    }

    #[tool(description = "Reactivate a stopped agent. Requires --allow-dangerous.")]
    async fn bwoc_start(
        &self,
        Parameters(NameArg { name }): Parameters<NameArg>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_dangerous()?;
        self.json_tool(&["start", &name, "--yes"]).await
    }

    #[tool(description = "Pause an agent (status = stopped). Requires --allow-dangerous.")]
    async fn bwoc_stop(
        &self,
        Parameters(NameArg { name }): Parameters<NameArg>,
    ) -> Result<CallToolResult, McpError> {
        self.gate_dangerous()?;
        self.json_tool(&["stop", &name, "--yes"]).await
    }

    // ---- internals ---------------------------------------------------------

    async fn json_tool(&self, args: &[&str]) -> Result<CallToolResult, McpError> {
        match self.bridge.json(args).await {
            Ok(v) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&v).unwrap_or_else(|_| v.to_string()),
            )])),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    async fn json_owned(&self, args: Vec<String>) -> Result<CallToolResult, McpError> {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.json_tool(&refs).await
    }

    async fn text_tool(&self, args: &[&str]) -> Result<CallToolResult, McpError> {
        match self.bridge.text(args).await {
            Ok(s) => Ok(CallToolResult::success(vec![Content::text(s)])),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    fn gate_write(&self) -> Result<(), McpError> {
        self.gate(self.posture.write, "--allow-write")
    }
    fn gate_exec(&self) -> Result<(), McpError> {
        self.gate(self.posture.exec, "--allow-exec")
    }
    fn gate_dangerous(&self) -> Result<(), McpError> {
        self.gate(self.posture.dangerous, "--allow-dangerous")
    }
    fn gate(&self, enabled: bool, flag: &str) -> Result<(), McpError> {
        if enabled {
            Ok(())
        } else {
            Err(McpError::invalid_request(
                format!("this tool tier is disabled; start bwoc-mcp with {flag}"),
                None,
            ))
        }
    }
}

#[tool_handler]
impl ServerHandler for BwocMcp {
    fn get_info(&self) -> ServerInfo {
        // ServerInfo is #[non_exhaustive] — mutate a Default rather than use a
        // struct literal.
        let mut info = ServerInfo::default();
        info.instructions = Some(
            "BWOC workspace control surface. Read tools (list, status, info, fleet, sessions, \
             trust, team/task lists, inbox, memory, peer) are always available; send/run, task \
             mutations, team/memory writes, and lifecycle tools (new/retire/start/stop) require \
             the operator to have enabled the matching --allow-* tier."
                .into(),
        );
        info.capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .enable_prompts()
            .build();
        // Identify as bwoc-mcp (name/version from this crate's Cargo env) — the
        // rmcp default / from_build_env() would report rmcp's own identity.
        let mut imp = Implementation::from_build_env();
        imp.name = env!("CARGO_PKG_NAME").into();
        imp.version = env!("CARGO_PKG_VERSION").into();
        info.server_info = imp;
        info
    }

    // ---- resources: read-only workspace context --------------------------

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let resources = vec![
            RawResource::new("bwoc://agents", "agents").no_annotation(),
            RawResource::new("bwoc://fleet", "fleet").no_annotation(),
            RawResource::new("bwoc://info", "info").no_annotation(),
        ];
        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let args: &[&str] = match request.uri.as_str() {
            "bwoc://agents" => &["list"],
            "bwoc://fleet" => &["fleet"],
            "bwoc://info" => &["info"],
            other => {
                return Err(McpError::resource_not_found(
                    format!("unknown resource: {other}"),
                    None,
                ));
            }
        };
        let value = self
            .bridge
            .json(args)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
        Ok(ReadResourceResult::new(vec![ResourceContents::text(
            text,
            request.uri,
        )]))
    }

    // ---- prompts: reusable BWOC workflow templates -----------------------

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        let prompts = vec![
            Prompt::new(
                "delegate",
                Some("Delegate a task to a BWOC agent via bwoc_run."),
                Some(vec![
                    PromptArgument::new("agent")
                        .with_description("Agent name or id to delegate to.")
                        .with_required(true),
                    PromptArgument::new("task")
                        .with_description("The task prompt.")
                        .with_required(true),
                ]),
            ),
            Prompt::new(
                "fleet_review",
                Some("Review fleet health and summarize risks."),
                None,
            ),
        ];
        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
            meta: None,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let args = request.arguments.unwrap_or_default();
        let messages = match request.name.as_str() {
            "delegate" => {
                let agent = args
                    .get("agent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<agent>");
                let task = args.get("task").and_then(|v| v.as_str()).unwrap_or("<task>");
                vec![PromptMessage::new_text(
                    PromptMessageRole::User,
                    format!(
                        "Delegate this task to BWOC agent `{agent}` by calling the `bwoc_run` \
                         tool (agent=\"{agent}\", task=\"{task}\"), then summarize the result.",
                    ),
                )]
            }
            "fleet_review" => vec![PromptMessage::new_text(
                PromptMessageRole::User,
                "Call `bwoc_fleet`, then summarize the fleet's health signals and flag any \
                 agent that is failing an Aparihāniya-dhamma signal."
                    .to_string(),
            )],
            other => {
                return Err(McpError::invalid_params(
                    format!("unknown prompt: {other}"),
                    None,
                ));
            }
        };
        Ok(GetPromptResult::new(messages))
    }
}
