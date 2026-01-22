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
                <tr class="border-b border-gray-700 hover:bg-gray-800" data-name="{}" data-branch="{}" data-status="{}">
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
                s.branch,
                s.status,
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
    <div class="flex h-screen">
        <!-- Sidebar Navigation -->
        <div class="w-64 bg-gray-800 border-r border-gray-700 overflow-y-auto">
            <div class="p-6">
                <h1 class="text-2xl font-bold mb-8">Scratchpad</h1>
                <nav class="space-y-4">
                    <a href="/" class="block px-4 py-2 bg-blue-600 rounded font-medium">Dashboard</a>
                    <a href="/scratches/create" class="block px-4 py-2 bg-green-600 hover:bg-green-700 rounded font-medium">+ Create Scratch</a>
                    <a href="/services" class="block px-4 py-2 bg-orange-600 hover:bg-orange-700 rounded font-medium">Services</a>
                    <a href="/config" class="block px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded font-medium">Configuration</a>
                    <button 
                        class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded font-medium"
                        hx-get="/"
                        hx-target="body"
                    >Refresh</button>
                </nav>
            </div>
        </div>

        <!-- Main Content -->
        <div class="flex-1 overflow-auto">
            <div class="container mx-auto px-8 py-8 max-w-6xl">
                <header class="mb-8">
                    <h2 class="text-3xl font-bold mb-2">Scratches</h2>
                    <p class="text-gray-400">Manage your scratch environments - {} total</p>
                </header>

                <!-- Search and Filter -->
                <div class="mb-6 bg-gray-800 rounded-lg p-4">
                    <input 
                        type="text" 
                        id="search-input"
                        placeholder="Search by name, branch, or status..."
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                    />
                    <div class="mt-3 flex space-x-2">
                        <button class="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded filter-btn" data-filter="all">All</button>
                        <button class="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded filter-btn" data-filter="running">Running</button>
                        <button class="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded filter-btn" data-filter="stopped">Stopped</button>
                    </div>
                </div>

                <!-- Scratches Table -->
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
                        <tbody id="scratches-tbody">
                            {}
                        </tbody>
                    </table>
                </div>

                <!-- Empty State -->
                <div id="empty-state" class="hidden text-center py-12">
                    <p class="text-gray-400 text-lg mb-4">No scratches found</p>
                    <a href="/scratches/create" class="px-4 py-2 bg-green-600 hover:bg-green-700 rounded font-medium">Create your first scratch</a>
                </div>

                <!-- Shared Services Section -->
                <div class="mt-8 bg-gray-800 rounded-lg p-6">
                    <h3 class="text-xl font-semibold mb-4">Shared Services</h3>
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
        </div>
    </div>

    <script>
        (function() {{
            const searchInput = document.getElementById('search-input');
            const filterBtns = document.querySelectorAll('.filter-btn');
            const tbody = document.getElementById('scratches-tbody');
            const emptyState = document.getElementById('empty-state');
            let currentFilter = 'all';

            function filterTable() {{
                const searchTerm = searchInput.value.toLowerCase();
                const rows = tbody.querySelectorAll('tr');
                let visibleCount = 0;

                rows.forEach(row => {{
                    let match = true;

                    // Search filter
                    if (searchTerm) {{
                        const name = row.dataset.name?.toLowerCase() || '';
                        const branch = row.dataset.branch?.toLowerCase() || '';
                        match = name.includes(searchTerm) || branch.includes(searchTerm);
                    }}

                    // Status filter
                    if (match && currentFilter !== 'all') {{
                        const status = row.dataset.status || '';
                        match = status === currentFilter;
                    }}

                    row.style.display = match ? '' : 'none';
                    if (match) visibleCount++;
                }});

                emptyState.style.display = visibleCount === 0 ? 'block' : 'none';
            }}

            searchInput.addEventListener('input', filterTable);

            filterBtns.forEach(btn => {{
                btn.addEventListener('click', () => {{
                    filterBtns.forEach(b => b.classList.remove('bg-blue-600'));
                    btn.classList.add('bg-blue-600');
                    currentFilter = btn.dataset.filter;
                    filterTable();
                }});
            }});

            // Set initial active filter
            filterBtns[0].classList.add('bg-blue-600');
        }})();
    </script>
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

/// Create scratch page - form for creating a new scratch
pub async fn create_scratch() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Create Scratch - Scratchpad</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
</head>
<body class="bg-gray-900 text-gray-100 min-h-screen">
    <div class="container mx-auto px-4 py-8 max-w-2xl">
        <header class="mb-8">
            <a href="/" class="text-blue-400 hover:underline mb-4 inline-block">&larr; Back to Dashboard</a>
            <h1 class="text-3xl font-bold mb-2">Create New Scratch</h1>
            <p class="text-gray-400">Set up a new scratch environment from a GitHub branch</p>
        </header>

        <div class="bg-gray-800 rounded-lg p-6">
            <form id="create-form">
                <div class="mb-6">
                    <label for="repo-url" class="block text-sm font-medium mb-2">GitHub Repository URL</label>
                    <input 
                        type="text" 
                        id="repo-url" 
                        name="repo_url"
                        placeholder="https://github.com/user/repo"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                        required
                    />
                    <p class="text-sm text-gray-400 mt-1">Full GitHub repository URL</p>
                </div>

                <div class="mb-6">
                    <label for="branch" class="block text-sm font-medium mb-2">Branch</label>
                    <div class="flex space-x-2">
                        <select 
                            id="branch" 
                            name="branch"
                            class="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                            required
                        >
                            <option value="">Select a branch...</option>
                        </select>
                        <button 
                            type="button"
                            id="refresh-branches"
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded"
                        >Refresh</button>
                    </div>
                    <p class="text-sm text-gray-400 mt-1">Choose the branch to use for this scratch</p>
                </div>

                <div class="mb-6">
                    <label for="name" class="block text-sm font-medium mb-2">Scratch Name (Optional)</label>
                    <input 
                        type="text" 
                        id="name" 
                        name="name"
                        placeholder="Defaults to branch name"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                        pattern="^[a-z0-9]+(?:-[a-z0-9]+)*$"
                    />
                    <p class="text-sm text-gray-400 mt-1">Lowercase alphanumeric and hyphens only. Defaults to branch name if empty.</p>
                </div>

                <div class="mb-6">
                    <label for="profile" class="block text-sm font-medium mb-2">Profile (Optional)</label>
                    <input 
                        type="text" 
                        id="profile" 
                        name="profile"
                        placeholder="default"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                    />
                    <p class="text-sm text-gray-400 mt-1">Docker Compose profile to use</p>
                </div>

                <div class="mb-6">
                    <label for="template" class="block text-sm font-medium mb-2">Template (Optional)</label>
                    <input 
                        type="text" 
                        id="template" 
                        name="template"
                        placeholder="default"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                    />
                    <p class="text-sm text-gray-400 mt-1">Template to use for this scratch</p>
                </div>

                <div class="flex space-x-4">
                    <button 
                        type="submit"
                        id="submit-btn"
                        class="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 rounded font-medium"
                    >Create Scratch</button>
                    <a href="/" class="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded font-medium text-center">Cancel</a>
                </div>
            </form>

            <div id="error-message" class="mt-4 p-4 bg-red-900 text-red-200 rounded hidden"></div>
        </div>
    </div>

    <script>
        (function() {
            const repoUrlInput = document.getElementById('repo-url');
            const branchSelect = document.getElementById('branch');
            const refreshBtn = document.getElementById('refresh-branches');
            const submitBtn = document.getElementById('submit-btn');
            const errorDiv = document.getElementById('error-message');
            const form = document.getElementById('create-form');

            function showError(msg) {
                errorDiv.textContent = msg;
                errorDiv.classList.remove('hidden');
            }

            function clearError() {
                errorDiv.classList.add('hidden');
                errorDiv.textContent = '';
            }

            async function fetchBranches() {
                const repoUrl = repoUrlInput.value.trim();
                if (!repoUrl) {
                    showError('Please enter a repository URL first');
                    return;
                }

                // Extract owner/repo from URL
                // Support formats like:
                // - https://github.com/user/repo
                // - https://github.com/user/repo.git
                // - git@github.com:user/repo.git
                let match = repoUrl.match(/(?:https:\/\/github\.com\/|git@github\.com:)([^\/]+)\/(.+?)(?:\.git)?$/);
                if (!match) {
                    showError('Invalid GitHub URL format');
                    return;
                }

                const owner = match[1];
                const repo = match[2];

                clearError();
                branchSelect.innerHTML = '<option value="">Loading branches...</option>';
                branchSelect.disabled = true;
                refreshBtn.disabled = true;

                try {
                    // Fetch branches from GitHub API
                    const response = await fetch(`https://api.github.com/repos/${owner}/${repo}/branches`);
                    if (!response.ok) {
                        throw new Error(`GitHub API error: ${response.status}`);
                    }

                    const branches = await response.json();
                    branchSelect.innerHTML = '<option value="">Select a branch...</option>';
                    
                    branches.forEach(branch => {
                        const option = document.createElement('option');
                        option.value = branch.name;
                        option.textContent = branch.name;
                        branchSelect.appendChild(option);
                    });

                    if (branches.length === 0) {
                        showError('No branches found in this repository');
                    }
                } catch (error) {
                    showError(`Failed to fetch branches: ${error.message}`);
                    branchSelect.innerHTML = '<option value="">Select a branch...</option>';
                } finally {
                    branchSelect.disabled = false;
                    refreshBtn.disabled = false;
                }
            }

            // Event listeners
            refreshBtn.addEventListener('click', fetchBranches);
            
            repoUrlInput.addEventListener('change', () => {
                clearError();
                branchSelect.innerHTML = '<option value="">Select a branch...</option>';
            });

            // Auto-fetch branches after user stops typing
            let debounceTimer;
            repoUrlInput.addEventListener('input', () => {
                clearTimeout(debounceTimer);
                debounceTimer = setTimeout(fetchBranches, 1000);
            });

            // Handle form submission
            form.addEventListener('submit', async (e) => {
                e.preventDefault();
                clearError();

                const branch = branchSelect.value;
                if (!branch) {
                    showError('Please select a branch');
                    return;
                }

                submitBtn.disabled = true;
                submitBtn.textContent = 'Creating...';

                const payload = {
                    branch: branch,
                    name: document.getElementById('name').value || undefined,
                    profile: document.getElementById('profile').value || undefined,
                    template: document.getElementById('template').value || undefined,
                };

                // Remove undefined fields
                Object.keys(payload).forEach(k => payload[k] === undefined && delete payload[k]);

                try {
                    const response = await fetch('/api/scratches', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify(payload),
                    });

                    const data = await response.json();

                    if (!response.ok) {
                        throw new Error(data.error || 'Failed to create scratch');
                    }

                    // Success - redirect to dashboard
                    window.location.href = '/';
                } catch (error) {
                    showError(`Error: ${error.message}`);
                    submitBtn.disabled = false;
                    submitBtn.textContent = 'Create Scratch';
                }
            });
        })();
    </script>
</body>
</html>
"#;
    Html(html.to_string())
}

/// Configuration editor page
pub async fn config_editor() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Configuration - Scratchpad</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-900 text-gray-100 min-h-screen">
    <div class="container mx-auto px-4 py-8 max-w-4xl">
        <header class="mb-8">
            <a href="/" class="text-blue-400 hover:underline mb-4 inline-block">&larr; Back to Dashboard</a>
            <h1 class="text-3xl font-bold mb-2">Configuration</h1>
            <p class="text-gray-400">Edit system configuration settings</p>
        </header>

        <div id="loading" class="bg-blue-900 text-blue-200 p-4 rounded mb-6">
            Loading configuration...
        </div>

        <div id="error-message" class="bg-red-900 text-red-200 p-4 rounded mb-6 hidden"></div>

        <form id="config-form" class="hidden">
            <!-- Server Configuration -->
            <div class="bg-gray-800 rounded-lg p-6 mb-6">
                <h2 class="text-xl font-semibold mb-4">Server Configuration</h2>
                
                <div class="mb-4">
                    <label for="server-host" class="block text-sm font-medium mb-2">Host</label>
                    <input 
                        type="text" 
                        id="server-host" 
                        name="server.host"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    />
                </div>

                <div class="mb-4">
                    <label for="server-port" class="block text-sm font-medium mb-2">Port</label>
                    <input 
                        type="number" 
                        id="server-port" 
                        name="server.port"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    />
                </div>

                <div class="mb-4">
                    <label for="server-releases-dir" class="block text-sm font-medium mb-2">Releases Directory</label>
                    <input 
                        type="text" 
                        id="server-releases-dir" 
                        name="server.releases_dir"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    />
                </div>
            </div>

            <!-- Docker Configuration -->
            <div class="bg-gray-800 rounded-lg p-6 mb-6">
                <h2 class="text-xl font-semibold mb-4">Docker Configuration</h2>
                
                <div class="mb-4">
                    <label for="docker-socket" class="block text-sm font-medium mb-2">Docker Socket</label>
                    <input 
                        type="text" 
                        id="docker-socket" 
                        name="docker.socket"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    />
                    <p class="text-sm text-gray-400 mt-1">Path to Docker socket or TCP endpoint</p>
                </div>

                <div class="mb-4">
                    <label for="docker-label-prefix" class="block text-sm font-medium mb-2">Label Prefix</label>
                    <input 
                        type="text" 
                        id="docker-label-prefix" 
                        name="docker.label_prefix"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    />
                </div>
            </div>

            <!-- Nginx Configuration -->
            <div class="bg-gray-800 rounded-lg p-6 mb-6">
                <h2 class="text-xl font-semibold mb-4">Nginx Configuration</h2>
                
                <div class="mb-4">
                    <label class="flex items-center">
                        <input 
                            type="checkbox" 
                            id="nginx-enabled" 
                            name="nginx.enabled"
                            class="mr-2"
                        />
                        <span class="text-sm font-medium">Enable Nginx Proxy</span>
                    </label>
                </div>

                <div class="mb-4">
                    <label for="nginx-domain" class="block text-sm font-medium mb-2">Domain</label>
                    <input 
                        type="text" 
                        id="nginx-domain" 
                        name="nginx.domain"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    />
                </div>

                <div class="mb-4">
                    <label for="nginx-routing" class="block text-sm font-medium mb-2">Routing Mode</label>
                    <select 
                        id="nginx-routing" 
                        name="nginx.routing"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    >
                        <option value="subdomain">Subdomain</option>
                        <option value="path">Path</option>
                    </select>
                </div>

                <div class="mb-4">
                    <label for="nginx-config-path" class="block text-sm font-medium mb-2">Config Path</label>
                    <input 
                        type="text" 
                        id="nginx-config-path" 
                        name="nginx.config_path"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    />
                </div>
            </div>

            <!-- GitHub Configuration -->
            <div class="bg-gray-800 rounded-lg p-6 mb-6">
                <h2 class="text-xl font-semibold mb-4">GitHub Configuration</h2>
                
                <div class="mb-4">
                    <label for="github-token" class="block text-sm font-medium mb-2">GitHub Token (Optional)</label>
                    <input 
                        type="password" 
                        id="github-token" 
                        name="github.token"
                        placeholder="Leave blank to keep current value"
                        class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
                    />
                    <p class="text-sm text-gray-400 mt-1">Personal access token for GitHub API</p>
                </div>
            </div>

            <div class="flex space-x-4">
                <button 
                    type="submit"
                    class="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 rounded font-medium"
                >Save Configuration</button>
                <a href="/" class="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded font-medium text-center">Cancel</a>
            </div>
        </form>
    </div>

    <script>
        (function() {
            const form = document.getElementById('config-form');
            const loadingDiv = document.getElementById('loading');
            const errorDiv = document.getElementById('error-message');

            function showError(msg) {
                errorDiv.textContent = msg;
                errorDiv.classList.remove('hidden');
            }

            function clearError() {
                errorDiv.classList.add('hidden');
            }

            async function loadConfig() {
                try {
                    const response = await fetch('/api/config');
                    if (!response.ok) {
                        throw new Error('Failed to load configuration');
                    }

                    const data = await response.json();
                    if (!data.success) {
                        throw new Error(data.error || 'Failed to load configuration');
                    }

                    const config = data.data;
                    populateForm(config);
                    
                    loadingDiv.classList.add('hidden');
                    form.classList.remove('hidden');
                } catch (error) {
                    showError(`Failed to load configuration: ${error.message}`);
                    loadingDiv.classList.add('hidden');
                }
            }

            function populateForm(config) {
                // Server config
                document.getElementById('server-host').value = config.server?.host || '';
                document.getElementById('server-port').value = config.server?.port || '';
                document.getElementById('server-releases-dir').value = config.server?.releases_dir || '';

                // Docker config
                document.getElementById('docker-socket').value = config.docker?.socket || '';
                document.getElementById('docker-label-prefix').value = config.docker?.label_prefix || '';

                // Nginx config
                document.getElementById('nginx-enabled').checked = config.nginx?.enabled ?? true;
                document.getElementById('nginx-domain').value = config.nginx?.domain || '';
                document.getElementById('nginx-routing').value = config.nginx?.routing || 'subdomain';
                document.getElementById('nginx-config-path').value = config.nginx?.config_path || '';
            }

            form.addEventListener('submit', async (e) => {
                e.preventDefault();
                clearError();

                const formData = new FormData(form);
                const config = {
                    server: {
                        host: formData.get('server.host'),
                        port: parseInt(formData.get('server.port')),
                        releases_dir: formData.get('server.releases_dir'),
                    },
                    docker: {
                        socket: formData.get('docker.socket'),
                        label_prefix: formData.get('docker.label_prefix'),
                    },
                    nginx: {
                        enabled: formData.get('nginx.enabled') === 'on',
                        domain: formData.get('nginx.domain'),
                        routing: formData.get('nginx.routing'),
                        config_path: formData.get('nginx.config_path'),
                    },
                };

                // Only include GitHub config if token is provided
                const githubToken = formData.get('github.token');
                if (githubToken) {
                    config.github = {
                        token: githubToken,
                    };
                }

                const submitBtn = form.querySelector('button[type="submit"]');
                submitBtn.disabled = true;
                submitBtn.textContent = 'Saving...';

                try {
                    const response = await fetch('/api/config', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify(config),
                    });

                    const data = await response.json();

                    if (!response.ok) {
                        throw new Error(data.error || 'Failed to save configuration');
                    }

                    // Success
                    showError('Configuration saved successfully!');
                    errorDiv.classList.remove('bg-red-900', 'text-red-200');
                    errorDiv.classList.add('bg-green-900', 'text-green-200');
                    
                    setTimeout(() => {
                        window.location.href = '/';
                    }, 2000);
                } catch (error) {
                    showError(`Error: ${error.message}`);
                    submitBtn.disabled = false;
                    submitBtn.textContent = 'Save Configuration';
                }
            });

            loadConfig();
        })();
    </script>
</body>
</html>
"#;
    Html(html.to_string())
}

/// Service management page
pub async fn service_manager(State(state): State<SharedState>) -> Html<String> {
    let state = state.read().await;
    
    // Get list of configured services
    let services_list: Vec<String> = state.config.services.keys().cloned().collect();
    
    let services_html: String = services_list
        .iter()
        .map(|service_name| {
            let service_config = &state.config.services[service_name];
            format!(
                r#"
                <tr class="border-b border-gray-700 hover:bg-gray-800">
                    <td class="px-4 py-3">{}</td>
                    <td class="px-4 py-3">{}</td>
                    <td class="px-4 py-3">
                        <span class="service-status text-gray-400" data-service="{}">Loading...</span>
                    </td>
                    <td class="px-4 py-3">
                        <div class="flex space-x-2">
                            <button 
                                class="px-2 py-1 text-xs bg-green-600 hover:bg-green-700 rounded service-start"
                                data-service="{}"
                            >Start</button>
                            <button 
                                class="px-2 py-1 text-xs bg-red-600 hover:bg-red-700 rounded service-stop"
                                data-service="{}"
                            >Stop</button>
                        </div>
                    </td>
                </tr>
                "#,
                service_name,
                service_config.image,
                service_name,
                service_name,
                service_name
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
    <title>Services - Scratchpad</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-900 text-gray-100 min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <header class="mb-8">
            <a href="/" class="text-blue-400 hover:underline mb-4 inline-block">&larr; Back to Dashboard</a>
            <h1 class="text-3xl font-bold mb-2">Shared Services</h1>
            <p class="text-gray-400">Manage shared infrastructure services</p>
        </header>

        <div id="error-message" class="bg-red-900 text-red-200 p-4 rounded mb-6 hidden"></div>

        <div class="bg-gray-800 rounded-lg overflow-hidden shadow-xl">
            <div class="p-6 border-b border-gray-700">
                <div class="flex space-x-4">
                    <button 
                        id="start-all"
                        class="px-4 py-2 bg-green-600 hover:bg-green-700 rounded font-medium"
                    >Start All Services</button>
                    <button 
                        id="stop-all"
                        class="px-4 py-2 bg-red-600 hover:bg-red-700 rounded font-medium"
                    >Stop All Services</button>
                </div>
            </div>

            <table class="w-full">
                <thead class="bg-gray-700">
                    <tr>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Service Name</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Image</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Status</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>
    </div>

    <script>
        (function() {{
            const errorDiv = document.getElementById('error-message');

            function showError(msg) {{
                errorDiv.textContent = msg;
                errorDiv.classList.remove('hidden');
                setTimeout(() => errorDiv.classList.add('hidden'), 5000);
            }}

            async function loadServiceStatus() {{
                try {{
                    const response = await fetch('/api/services');
                    const data = await response.json();
                    if (data.success && data.data) {{
                        Object.entries(data.data).forEach(([service, status]) => {{
                            const statusEl = document.querySelector(`.service-status[data-service="${{service}}"]`);
                            if (statusEl) {{
                                statusEl.textContent = status;
                                statusEl.className = `service-status ${{status === 'running' ? 'text-green-500' : 'text-red-500'}}`;
                            }}
                        }});
                    }}
                }} catch (error) {{
                    console.error('Failed to load service status:', error);
                }}
            }}

            async function startService(service) {{
                try {{
                    const response = await fetch(`/api/services/${{service}}/start`, {{
                        method: 'POST',
                    }});
                    if (response.ok) {{
                        setTimeout(loadServiceStatus, 1000);
                    }} else {{
                        showError(`Failed to start ${{service}}`);
                    }}
                }} catch (error) {{
                    showError(`Error starting ${{service}}: ${{error.message}}`);
                }}
            }}

            async function stopService(service) {{
                try {{
                    const response = await fetch(`/api/services/${{service}}/stop`, {{
                        method: 'POST',
                    }});
                    if (response.ok) {{
                        setTimeout(loadServiceStatus, 1000);
                    }} else {{
                        showError(`Failed to stop ${{service}}`);
                    }}
                }} catch (error) {{
                    showError(`Error stopping ${{service}}: ${{error.message}}`);
                }}
            }}

            // Event listeners
            document.getElementById('start-all').addEventListener('click', async () => {{
                const response = await fetch('/api/services/start', {{ method: 'POST' }});
                if (response.ok) {{
                    setTimeout(loadServiceStatus, 1000);
                }} else {{
                    showError('Failed to start all services');
                }}
            }});

            document.getElementById('stop-all').addEventListener('click', async () => {{
                const response = await fetch('/api/services/stop', {{ method: 'POST' }});
                if (response.ok) {{
                    setTimeout(loadServiceStatus, 1000);
                }} else {{
                    showError('Failed to stop all services');
                }}
            }});

            document.querySelectorAll('.service-start').forEach(btn => {{
                btn.addEventListener('click', () => startService(btn.dataset.service));
            }});

            document.querySelectorAll('.service-stop').forEach(btn => {{
                btn.addEventListener('click', () => stopService(btn.dataset.service));
            }});

            // Initial load
            loadServiceStatus();
            
            // Refresh every 5 seconds
            setInterval(loadServiceStatus, 5000);
        }})();
    </script>
</body>
</html>
        "#,
        services_html
    );

    Html(html)
}
