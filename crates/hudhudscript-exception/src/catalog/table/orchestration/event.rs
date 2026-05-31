use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const EVENT_BUS_CHANNEL_CLOSED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(78),
        long_code: "HHS_E_EVENT_BUS_CHANNEL_CLOSED",
        short_code: "E0078",
        title: "Event bus channel is closed",
        short_description: "A publish or subscribe operation was attempted on an event bus channel that has already been closed.",
        long_description: "Event bus channels in HudHudScript orchestration are backed by async broadcast primitives that can be shut down either explicitly or when the owning network is torn down. Once a channel is closed, no further messages can be sent and no new subscribers can attach.

This error usually indicates that the orchestrator was shut down while background tasks still held references to the bus, or that a subscriber outlived the publisher. Ensure clean shutdown ordering: tear down producers after all consumers have drained, or use `bus.is_closed()` guards before publishing.

In long-running deployments, consider wrapping publishes in a retry-with-reconnect strategy if channels may be recreated.",
        hints: &["Check `bus.is_closed()` before publishing events", "Ensure the event bus outlives all its producers and subscribers", "Shut down subscribers before closing the channel to avoid late sends"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_EVENT_BUS_NO_SUBSCRIBERS", "HHS_E_ORCHESTRATION_NETWORK_ERROR"],
        since_version: "0.4.0",
        category: ExceptionCategory::Orchestration,
    };

pub const EVENT_BUS_NO_SUBSCRIBERS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(79),
        long_code: "HHS_E_EVENT_BUS_NO_SUBSCRIBERS",
        short_code: "E0079",
        title: "No subscribers attached to event bus topic",
        short_description: "A publish operation found no active subscribers on the target topic, and the bus is configured to treat this as an error.",
        long_description: "By default, event buses are fire-and-forget: publishing to an empty topic silently drops the message. When a bus is configured in strict mode (or when using `publish_required`), dispatching to a topic with zero subscribers raises this error so unnoticed drops cannot propagate.

To fix this, either register at least one subscriber before the first publish, switch the bus to best-effort mode, or guard the call with `bus.subscriber_count(topic) > 0`. Race conditions at startup are a common cause — producers can come online before consumers finish subscribing.

If the topic is genuinely optional, use the non-strict publish variant instead of treating empty delivery as an error.",
        hints: &["Use a startup barrier so subscribers attach before producers begin", "Call `bus.subscriber_count(topic)` to gate strict publishes", "Switch to best-effort publish if empty delivery is acceptable"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_EVENT_BUS_CHANNEL_CLOSED"],
        since_version: "0.4.0",
        category: ExceptionCategory::Orchestration,
    };
