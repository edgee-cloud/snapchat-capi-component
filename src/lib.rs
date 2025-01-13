mod snapchat_payload;

use std::collections::HashMap;

use crate::exports::edgee::protocols::provider::{
    Data, Dict, EdgeeRequest, Event, Guest, HttpMethod,
};
use snapchat_payload::{parse_value, SnapchatEvent, SnapchatPayload};

wit_bindgen::generate!({world: "data-collection", path: "wit", with: { "edgee:protocols/provider": generate }});

export!(SnapchatComponent);

struct SnapchatComponent;

impl Guest for SnapchatComponent {
    fn page(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Page(ref data) = edgee_event.data {
            let mut snapchat_payload = SnapchatPayload::new(cred_map).map_err(|e| e.to_string())?;

            let mut event =
                SnapchatEvent::new(&edgee_event, "PAGE_VIEW").map_err(|e| e.to_string())?;

            // Create custom data
            let mut custom_data: HashMap<String, serde_json::Value> = HashMap::new();

            if !data.name.is_empty() {
                custom_data.insert("page_name".to_string(), parse_value(&data.name));
            }
            if !data.category.is_empty() {
                custom_data.insert("page_category".to_string(), parse_value(&data.category));
            }
            if !data.title.is_empty() {
                custom_data.insert("page_title".to_string(), parse_value(&data.title));
            }

            // Add custom properties from page data
            for (key, value) in data.properties.iter() {
                custom_data.insert(key.clone(), parse_value(value));
            }

            event.custom_data = Some(custom_data);
            snapchat_payload.data.push(event);

            Ok(build_edgee_request(snapchat_payload))
        } else {
            Err("Missing page data".to_string())
        }
    }

    fn track(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(ref data) = edgee_event.data {
            if data.name.is_empty() {
                return Err("Track name is not set".to_string());
            }

            let mut snapchat_payload = SnapchatPayload::new(cred_map).map_err(|e| e.to_string())?;
            let mut event =
                SnapchatEvent::new(&edgee_event, data.name.as_str()).map_err(|e| e.to_string())?;

            // Create custom data from properties
            let mut custom_data: HashMap<String, serde_json::Value> = HashMap::new();
            for (key, value) in data.properties.iter() {
                custom_data.insert(key.clone(), parse_value(value));
            }
            event.custom_data = Some(custom_data);
            snapchat_payload.data.push(event);

            Ok(build_edgee_request(snapchat_payload))
        } else {
            Err("Missing track data".to_string())
        }
    }

    fn user(_edgee_event: Event, _cred_map: Dict) -> Result<EdgeeRequest, String> {
        Err("User event not implemented for this component".to_string())
    }
}

fn build_edgee_request(snapchat_payload: SnapchatPayload) -> EdgeeRequest {
    let headers = vec![(
        String::from("content-type"),
        String::from("application/json"),
    )];

    let url = format!(
        "https://tr.snapchat.com/v3/{}/events?access_token={}",
        snapchat_payload.pixel_id, snapchat_payload.access_token
    );

    let url = if let Some(test_code) = snapchat_payload.test_event_code.clone() {
        format!("{}&test_event_code={}", url, test_code)
    } else {
        url
    };

    EdgeeRequest {
        method: HttpMethod::Post,
        url,
        headers,
        body: serde_json::to_string(&snapchat_payload).unwrap(),
    }
}
