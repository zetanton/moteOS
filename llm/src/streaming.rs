extern crate alloc;

use alloc::string::String;

/// Iterate Server-Sent Events payloads (`data: ...`) from a response body.
///
/// This supports multi-line `data:` fields and dispatches an event when a blank line is reached.
pub fn for_each_sse_data(body: &str, mut on_data: impl FnMut(&str)) {
    let mut data_buf = String::new();

    for raw_line in body.lines() {
        let line = raw_line.trim_end_matches('\r');

        if line.is_empty() {
            if !data_buf.is_empty() {
                let data = data_buf.trim_end_matches('\n');
                on_data(data);
                data_buf.clear();
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("data:") {
            let payload = rest.trim_start();
            data_buf.push_str(payload);
            data_buf.push('\n');
        }
    }

    if !data_buf.is_empty() {
        let data = data_buf.trim_end_matches('\n');
        on_data(data);
    }
}
