use crate::types::*;

pub fn print_send_result(resp: &NotifyResponse, json: bool) {
    if json {
        println!("{}", serde_json::to_string(resp).unwrap_or_default());
    } else {
        eprintln!("Sent: {}", resp.id);
    }
}

pub fn print_health(resp: &HealthResponse) {
    println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
}

pub fn print_active(notifications: &[serde_json::Value]) {
    println!("{}", serde_json::to_string_pretty(notifications).unwrap_or_default());
}

pub fn print_dismiss_all(resp: &DismissAllResponse, json: bool) {
    if json {
        println!("{}", serde_json::to_string(resp).unwrap_or_default());
    } else {
        eprintln!("Dismissed {} notification(s)", resp.dismissed);
    }
}

pub fn print_action_result(resp: &WebhookResult) {
    println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
}
