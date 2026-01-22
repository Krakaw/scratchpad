//! Web UI handlers

use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::api::server::SharedState;
use crate::scratch;

/// Dashboard page - lists all scratches
pub async fn dashboard(State(state): State<SharedState>) -> Html<String> {
    let state = state.read().await;

    let scratches = scratch::list_scratches(&state.config, &state.docker)
        .await
        .unwrap_or_default();

    let scratch_rows: String = scratches
        .iter()
        .map(|s| {
            let status_class = match s.status.as_str() {
                "running" => "text-green-500",
                "stopped" => "text-red-500",
                _ => "text-yellow-500",
            };

            let services: String = s
                .services
                .iter()
                .map(|(name, status)| {
                    let icon = if status == "running" { "●" } else { "○" };
                    format!("<span class=\"mr-2\">{} {}</span>", icon, name)
                })
                .collect();

            format!(
                r#"
                <tr class="border-b border-gray-700 hover:bg-gray-800">
                    <td class="px-4 py-3">
                        <a href="/scratches/{}" class="text-blue-400 hover:underline">{}</a>
                    </td>
                    <td class="px-4 py-3">{}</td>
                    <td class="px-4 py-3 {}">
                        <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-700">
                            {}
                        </span>
                    </td>
                    <td class="px-4 py-3 text-sm">{}</td>
                    <td class="px-4 py-3">
                        <a href="{}" target="_blank" class="text-blue-400 hover:underline text-sm">{}</a>
                    </td>
                    <td class="px-4 py-3">
                        <div class="flex space-x-2" hx-target="closest tr" hx-swap="outerHTML">
                            {}
                            <button 
                                class="px-2 py-1 text-xs bg-red-600 hover:bg-red-700 rounded"
                                hx-delete="/api/scratches/{}"
                                hx-confirm="Are you sure you want to delete this scratch?"
                            >Delete</button>
                        </div>
                    </td>
                </tr>
                "#,
                s.name,
                s.name,
                s.branch,
                status_class,
                s.status,
                services,
                s.url.as_deref().unwrap_or("#"),
                s.url.as_deref().unwrap_or("-"),
                if s.status == "running" {
                    format!(r#"<button class="px-2 py-1 text-xs bg-yellow-600 hover:bg-yellow-700 rounded" hx-post="/api/scratches/{}/stop">Stop</button>"#, s.name)
                } else {
                    format!(r#"<button class="px-2 py-1 text-xs bg-green-600 hover:bg-green-700 rounded" hx-post="/api/scratches/{}/start">Start</button>"#, s.name)
                },
                s.name
            )
        })
        .collect();

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Scratchpad</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body class="bg-gray-900 text-gray-100 min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <header class="mb-8">
            <h1 class="text-3xl font-bold mb-2">Scratchpad</h1>
            <p class="text-gray-400">Manage your scratch environments</p>
        </header>

        <div class="mb-6 flex justify-between items-center">
            <div>
                <span class="text-sm text-gray-400">{} scratches</span>
            </div>
            <div class="flex space-x-4">
                <button 
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded font-medium"
                    hx-get="/"
                    hx-target="body"
                >Refresh</button>
            </div>
        </div>

        <div class="bg-gray-800 rounded-lg overflow-hidden shadow-xl">
            <table class="w-full">
                <thead class="bg-gray-700">
                    <tr>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Name</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Branch</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Status</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Services</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold">URL</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>

        <div class="mt-8 bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4">Shared Services</h2>
            <div class="flex space-x-4">
                <button 
                    class="px-4 py-2 bg-green-600 hover:bg-green-700 rounded"
                    hx-post="/api/services/start"
                >Start All</button>
                <button 
                    class="px-4 py-2 bg-red-600 hover:bg-red-700 rounded"
                    hx-post="/api/services/stop"
                >Stop All</button>
            </div>
        </div>
    </div>
</body>
</html>
        "#,
        scratches.len(),
        scratch_rows
    );

    Html(html)
}

/// Scratch detail page
pub async fn scratch_detail(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> Html<String> {
    let state = state.read().await;

    let scratch_status = scratch::get_scratch_status(&state.config, &state.docker, &name)
        .await
        .unwrap_or_else(|_| scratch::ScratchStatus::new(name.clone(), "unknown".to_string()));

    let services_html: String = scratch_status
        .services
        .iter()
        .map(|(name, status)| {
            let status_class = if status == "running" {
                "text-green-500"
            } else {
                "text-red-500"
            };
            format!(
                r#"
                <div class="flex justify-between items-center py-2 border-b border-gray-700">
                    <span>{}</span>
                    <span class="{}">{}</span>
                </div>
                "#,
                name, status_class, status
            )
        })
        .collect();

    let databases_html: String = scratch_status
        .databases
        .iter()
        .map(|db| format!("<li class=\"py-1\">{}</li>", db))
        .collect();

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Scratchpad</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body class="bg-gray-900 text-gray-100 min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <header class="mb-8">
            <a href="/" class="text-blue-400 hover:underline mb-4 inline-block">&larr; Back to Dashboard</a>
            <h1 class="text-3xl font-bold">{}</h1>
            <p class="text-gray-400">Branch: {}</p>
        </header>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="bg-gray-800 rounded-lg p-6">
                <h2 class="text-xl font-semibold mb-4">Status</h2>
                <div class="mb-4">
                    <span class="text-lg font-medium {}">{}</span>
                </div>
                <div class="flex space-x-4">
                    {}
                    <button 
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded"
                        hx-post="/api/scratches/{}/restart"
                    >Restart</button>
                </div>
            </div>

            <div class="bg-gray-800 rounded-lg p-6">
                <h2 class="text-xl font-semibold mb-4">Services</h2>
                <div class="space-y-1">
                    {}
                </div>
            </div>

            <div class="bg-gray-800 rounded-lg p-6">
                <h2 class="text-xl font-semibold mb-4">Databases</h2>
                <ul class="list-disc list-inside text-gray-300">
                    {}
                </ul>
            </div>

            <div class="bg-gray-800 rounded-lg p-6">
                <h2 class="text-xl font-semibold mb-4">Access</h2>
                <a href="{}" target="_blank" class="text-blue-400 hover:underline">{}</a>
            </div>
        </div>

        <div class="mt-8 bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4">Logs</h2>
            <div 
                id="logs-container"
                class="bg-gray-900 rounded p-4 font-mono text-sm h-96 overflow-auto"
            >
                <div class="text-gray-500">Loading initial logs...</div>
            </div>
            <script>
                (function() {{
                    const container = document.getElementById('logs-container');
                    const scratchName = '{}';
                    let ws = null;
                    let pingInterval = null;
                    
                    function displayLog(timestamp, service, line) {{
                        const time_str = new Date(timestamp).toLocaleTimeString();
                        const service_str = service ? ` [${{service}}]` : '';
                        const logDiv = document.createElement('div');
                        logDiv.textContent = `${{time_str}}${{service_str}}: ${{line}}`;
                        logDiv.className = 'text-gray-300';
                        container.appendChild(logDiv);
                        
                        // Keep only last 500 lines for performance
                        const lines = container.querySelectorAll('div:not(:first-child)');
                        if (lines.length > 500) {{
                            lines[0].remove();
                        }}
                        
                        // Auto-scroll to bottom
                        container.scrollTop = container.scrollHeight;
                    }}
                    
                    function fetchInitialLogs() {{
                        fetch(`/api/scratches/${{scratchName}}/logs?tail=200`)
                            .then(r => r.json())
                            .then(data => {{
                                if (data.success && data.data) {{
                                    container.innerHTML = '';
                                    data.data.forEach(line => {{
                                        const logDiv = document.createElement('div');
                                        logDiv.textContent = line;
                                        logDiv.className = 'text-gray-300';
                                        container.appendChild(logDiv);
                                    }});
                                }}
                            }})
                            .catch(e => {{
                                console.error('Failed to fetch initial logs:', e);
                                container.innerHTML = '<div class="text-red-500">Failed to load initial logs</div>';
                            }});
                    }}
                    
                    function connectWebSocket() {{
                        ws = new WebSocket(`${{window.location.protocol === 'https:' ? 'wss:' : 'ws:'}}//${{window.location.host}}/ws`);
                        
                        ws.onopen = function() {{
                            console.log('WebSocket connected');
                            // Subscribe to logs for this scratch
                            const msg = {{
                                type: 'Subscribe',
                                channels: [`logs:${{scratchName}}`]
                            }};
                            ws.send(JSON.stringify(msg));
                            
                            // Start keep-alive pings
                            if (pingInterval) clearInterval(pingInterval);
                            pingInterval = setInterval(() => {{
                                if (ws && ws.readyState === WebSocket.OPEN) {{
                                    ws.send(JSON.stringify({{ type: 'Ping' }}));
                                }}
                            }}, 30000);
                        }};
                        
                        ws.onmessage = function(event) {{
                            const msg = JSON.parse(event.data);
                            
                            if (msg.type === 'Log') {{
                                displayLog(msg.timestamp, msg.service, msg.line);
                            }} else if (msg.type === 'Subscribed') {{
                                if (container.textContent.includes('Loading initial logs')) {{
                                    container.innerHTML = '<div class="text-gray-400">Waiting for logs...</div>';
                                }}
                            }} else if (msg.type === 'Error') {{
                                console.error('WebSocket error message:', msg.message);
                            }} else if (msg.type === 'Pong') {{
                                // Keep-alive response
                            }}
                        }};
                        
                        ws.onerror = function(error) {{
                            console.error('WebSocket error:', error);
                            const errorDiv = document.createElement('div');
                            errorDiv.textContent = 'Connection error. Attempting to reconnect...';
                            errorDiv.className = 'text-red-500';
                            container.appendChild(errorDiv);
                        }};
                        
                        ws.onclose = function() {{
                            console.log('WebSocket disconnected, attempting to reconnect in 3s...');
                            if (pingInterval) clearInterval(pingInterval);
                            // Attempt to reconnect after 3 seconds
                            setTimeout(connectWebSocket, 3000);
                        }};
                    }}
                    
                    // Load initial logs and connect WebSocket
                    fetchInitialLogs();
                    setTimeout(connectWebSocket, 500);
                    
                    // Cleanup on page unload
                    window.addEventListener('beforeunload', () => {{
                        if (ws) ws.close();
                        if (pingInterval) clearInterval(pingInterval);
                    }});
                }})();
            </script>
        </div>

        <div class="mt-8">
            <button 
                class="px-4 py-2 bg-red-600 hover:bg-red-700 rounded"
                hx-delete="/api/scratches/{}"
                hx-confirm="Are you sure you want to delete this scratch? This cannot be undone."
                hx-redirect="/"
            >Delete Scratch</button>
        </div>
    </div>
</body>
</html>
        "#,
        scratch_status.name,
        scratch_status.name,
        scratch_status.branch,
        if scratch_status.status == "running" { "text-green-500" } else { "text-red-500" },
        scratch_status.status,
        if scratch_status.status == "running" {
            format!(r#"<button class="px-4 py-2 bg-yellow-600 hover:bg-yellow-700 rounded" hx-post="/api/scratches/{}/stop">Stop</button>"#, scratch_status.name)
        } else {
            format!(r#"<button class="px-4 py-2 bg-green-600 hover:bg-green-700 rounded" hx-post="/api/scratches/{}/start">Start</button>"#, scratch_status.name)
        },
        scratch_status.name,
        services_html,
        if databases_html.is_empty() { "<li class=\"text-gray-500\">No databases</li>".to_string() } else { databases_html },
        scratch_status.url.as_deref().unwrap_or("#"),
        scratch_status.url.as_deref().unwrap_or("-"),
        scratch_status.name,
        scratch_status.name
    );

    Html(html)
}
