//! Webhook processing module for payment events
//! Handles webhooks from Lemon Squeezy, PayPal, Paddle, and Polygon

pub mod event_handler;

pub use event_handler::{
    WebhookProcessor,
    WebhookProvider,
    WebhookEventType,
    WebhookEvent,
    WebhookSecrets,
    ProcessingResult,
};
