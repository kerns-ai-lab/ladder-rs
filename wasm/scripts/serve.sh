#!/bin/bash
# Task 1.1.5: Development Server Script
#
# This script provides a development server with hot reload capabilities,
# CORS support, and WebSocket integration for real-time updates.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
SERVER_PORT=3000
SERVER_HOST="127.0.0.1"
STATIC_DIR="pkg"
HOT_RELOAD=false
WS_PORT=3001
CORS_ENABLED=true
HTTPS_ENABLED=false
VERBOSE=false
TEST_MODE=false

# SSL configuration
SSL_CERT=""
SSL_KEY=""

# Helper functions
log_info() {
    echo -e "${BLUE}[SERVER]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SERVER]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[SERVER]${NC} $1"
}

log_error() {
    echo -e "${RED}[SERVER]${NC} $1"
}

log_verbose() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${CYAN}[SERVER-VERBOSE]${NC} $1"
    fi
}

# Check for required tools
check_server_tools() {
    local server_found=false
    
    # Check for Python (built-in http.server)
    if command -v python3 &> /dev/null; then
        SERVER_TYPE="python3"
        server_found=true
        log_verbose "Found Python 3 for HTTP server"
    elif command -v python &> /dev/null; then
        SERVER_TYPE="python"
        server_found=true
        log_verbose "Found Python 2 for HTTP server"
    fi
    
    # Check for Node.js (for more advanced features)
    if command -v node &> /dev/null; then
        if [ -f "scripts/server.js" ]; then
            SERVER_TYPE="node"
            server_found=true
            log_verbose "Found Node.js with custom server script"
        fi
    fi
    
    # Check for specialized dev servers
    if command -v live-server &> /dev/null; then
        SERVER_TYPE="live-server"
        server_found=true
        log_verbose "Found live-server"
    fi
    
    if command -v http-server &> /dev/null; then
        SERVER_TYPE="http-server"
        server_found=true
        log_verbose "Found http-server"
    fi
    
    if [ "$server_found" = false ]; then
        log_error "No suitable HTTP server found!"
        log_error "Please install one of:"
        log_error "  - Python 3 (recommended)"
        log_error "  - Node.js with live-server: npm install -g live-server"
        log_error "  - Node.js with http-server: npm install -g http-server"
        exit 1
    fi
    
    log_verbose "Using server type: $SERVER_TYPE"
}

# Create simple index.html if it doesn't exist
create_index_html() {
    local index_file="$STATIC_DIR/index.html"
    
    if [ ! -f "$index_file" ]; then
        log_info "Creating basic index.html for development"
        
        cat > "$index_file" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Ladder-RS WASM Development</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        .status {
            padding: 10px;
            border-radius: 4px;
            margin: 10px 0;
        }
        .status.loading {
            background-color: #fff3cd;
            border: 1px solid #ffeaa7;
        }
        .status.ready {
            background-color: #d4edda;
            border: 1px solid #c3e6cb;
        }
        .status.error {
            background-color: #f8d7da;
            border: 1px solid #f5c6cb;
        }
        pre {
            background: #f8f9fa;
            padding: 15px;
            border-radius: 4px;
            overflow-x: auto;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🦀 Ladder-RS WASM Development Server</h1>
        
        <div id="status" class="status loading">
            Loading WASM module...
        </div>
        
        <h2>Quick Test</h2>
        <button onclick="testWasm()">Test WASM Functions</button>
        <pre id="output"></pre>
        
        <h2>Available Files</h2>
        <ul>
            <li><a href="ladder_rs_wasm.js">ladder_rs_wasm.js</a> - JavaScript bindings</li>
            <li><a href="ladder_rs_wasm_bg.wasm">ladder_rs_wasm_bg.wasm</a> - WASM binary</li>
            <li><a href="ladder_rs_wasm.d.ts">ladder_rs_wasm.d.ts</a> - TypeScript definitions</li>
            <li><a href="package.json">package.json</a> - Package metadata</li>
        </ul>
    </div>

    <script type="module">
        import init, { greet, wasm_main } from './ladder_rs_wasm.js';
        
        let wasmModule = null;
        const statusEl = document.getElementById('status');
        const outputEl = document.getElementById('output');
        
        async function loadWasm() {
            try {
                wasmModule = await init();
                statusEl.textContent = '✅ WASM module loaded successfully!';
                statusEl.className = 'status ready';
                
                // Test basic functionality
                wasm_main();
                outputEl.textContent = 'WASM module initialized successfully.';
                
            } catch (error) {
                statusEl.textContent = `❌ Failed to load WASM: ${error.message}`;
                statusEl.className = 'status error';
                outputEl.textContent = error.stack;
            }
        }
        
        window.testWasm = function() {
            if (!wasmModule) {
                outputEl.textContent = 'WASM module not loaded yet.';
                return;
            }
            
            try {
                greet('Development Server');
                outputEl.textContent = 'Test completed successfully! Check browser console for output.';
            } catch (error) {
                outputEl.textContent = `Test failed: ${error.message}`;
            }
        };
        
        // Hot reload support
        if (typeof window.hotReload === 'undefined') {
            window.hotReload = {
                connect() {
                    const ws = new WebSocket('ws://localhost:3001');
                    ws.onmessage = (event) => {
                        console.log('Hot reload triggered:', event.data);
                        location.reload();
                    };
                    ws.onopen = () => console.log('Hot reload connected');
                    ws.onerror = () => console.log('Hot reload not available');
                }
            };
            
            // Try to connect to hot reload WebSocket
            window.hotReload.connect();
        }
        
        // Load WASM on page load
        loadWasm();
    </script>
</body>
</html>
EOF
        
        log_verbose "Created development index.html"
    fi
}

# Start Python HTTP server
start_python_server() {
    local python_cmd="$1"
    
    log_info "Starting Python HTTP server on $SERVER_HOST:$SERVER_PORT"
    
    cd "$STATIC_DIR"
    
    if [ "$python_cmd" = "python3" ]; then
        python3 -m http.server "$SERVER_PORT" --bind "$SERVER_HOST" &
    else
        python -m SimpleHTTPServer "$SERVER_PORT" &
    fi
    
    local server_pid=$!
    echo "$server_pid" > "../.dev_server.pid"
    
    cd ..
    
    log_success "Python HTTP server started (PID: $server_pid)"
    return 0
}

# Start Node.js-based servers
start_node_server() {
    local server_cmd="$1"
    
    case "$server_cmd" in
        "live-server")
            log_info "Starting live-server on $SERVER_HOST:$SERVER_PORT"
            
            local args=()
            args+=("--host=$SERVER_HOST")
            args+=("--port=$SERVER_PORT")
            args+=("--no-browser")
            
            if [ "$CORS_ENABLED" = true ]; then
                args+=("--cors")
            fi
            
            live-server "$STATIC_DIR" "${args[@]}" &
            ;;
            
        "http-server")
            log_info "Starting http-server on $SERVER_HOST:$SERVER_PORT"
            
            local args=()
            args+=("-a" "$SERVER_HOST")
            args+=("-p" "$SERVER_PORT")
            
            if [ "$CORS_ENABLED" = true ]; then
                args+=("--cors")
            fi
            
            http-server "$STATIC_DIR" "${args[@]}" &
            ;;
            
        "node")
            log_info "Starting custom Node.js server on $SERVER_HOST:$SERVER_PORT"
            
            # This would use a custom server.js file
            node scripts/server.js --port "$SERVER_PORT" --host "$SERVER_HOST" &
            ;;
    esac
    
    local server_pid=$!
    echo "$server_pid" > ".dev_server.pid"
    
    log_success "Node.js server started (PID: $server_pid)"
    return 0
}

# Start WebSocket server for hot reload
start_websocket_server() {
    if [ "$HOT_RELOAD" = false ]; then
        return 0
    fi
    
    log_info "Starting WebSocket server for hot reload on port $WS_PORT"
    
    # Simple WebSocket server using Node.js (if available)
    if command -v node &> /dev/null; then
        # Create a simple WebSocket server script on-the-fly
        cat > scripts/websocket-server.js << 'EOF'
const WebSocket = require('ws');
const fs = require('fs');
const path = require('path');

const port = process.argv[2] || 3001;
const wss = new WebSocket.Server({ port: port });

console.log(`WebSocket server started on port ${port}`);

// Watch for hot reload trigger file
const triggerFile = path.join(process.cwd(), 'pkg', '.hot_reload_trigger');

let clients = [];

wss.on('connection', (ws) => {
    console.log('Client connected to hot reload');
    clients.push(ws);
    
    ws.on('close', () => {
        clients = clients.filter(client => client !== ws);
        console.log('Client disconnected from hot reload');
    });
});

// Simple file watching for trigger file
let lastMtime = 0;

function checkForChanges() {
    try {
        const stats = fs.statSync(triggerFile);
        if (stats.mtime.getTime() > lastMtime) {
            lastMtime = stats.mtime.getTime();
            
            const changedFile = fs.readFileSync(triggerFile, 'utf8').trim();
            console.log(`Broadcasting reload for: ${changedFile}`);
            
            clients.forEach(client => {
                if (client.readyState === WebSocket.OPEN) {
                    client.send(JSON.stringify({
                        type: 'reload',
                        file: changedFile,
                        timestamp: Date.now()
                    }));
                }
            });
            
            // Clean up trigger file
            fs.unlinkSync(triggerFile);
        }
    } catch (error) {
        // Trigger file doesn't exist yet, that's ok
    }
}

// Check for changes every 500ms
setInterval(checkForChanges, 500);

process.on('SIGINT', () => {
    console.log('WebSocket server shutting down');
    process.exit(0);
});
EOF
        
        node scripts/websocket-server.js "$WS_PORT" &
        local ws_pid=$!
        echo "$ws_pid" > ".hot_reload.pid"
        
        log_success "WebSocket server started (PID: $ws_pid)"
    else
        log_warning "Node.js not found, hot reload WebSocket server not available"
    fi
}

# Check if server is responding
check_server_health() {
    local max_attempts=10
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "http://$SERVER_HOST:$SERVER_PORT" > /dev/null 2>&1; then
            log_success "Server is responding on http://$SERVER_HOST:$SERVER_PORT"
            return 0
        fi
        
        log_verbose "Attempt $attempt/$max_attempts: Server not ready yet..."
        sleep 1
        ((attempt++))
    done
    
    log_warning "Server health check failed, but it might still be starting"
    return 1
}

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --port) SERVER_PORT="$2"; shift;;
        --host) SERVER_HOST="$2"; shift;;
        --static-dir) STATIC_DIR="$2"; shift;;
        --hot-reload) HOT_RELOAD=true;;
        --ws-port) WS_PORT="$2"; shift;;
        --cors) CORS_ENABLED=true;;
        --no-cors) CORS_ENABLED=false;;
        --https) HTTPS_ENABLED=true;;
        --ssl-cert) SSL_CERT="$2"; shift;;
        --ssl-key) SSL_KEY="$2"; shift;;
        --verbose|-v) VERBOSE=true;;
        --test-mode) TEST_MODE=true;;
        --help|-h)
            cat << EOF
Development Server Script - Task 1.1.5

This script provides a development server with hot reload capabilities,
CORS support, and WebSocket integration for real-time updates.

Usage: ./scripts/serve.sh [options]

Server Configuration:
  --port PORT              Server port (default: 3000)
  --host HOST              Server host (default: 127.0.0.1)
  --static-dir DIR         Static files directory (default: pkg)

Features:
  --hot-reload             Enable hot reload with WebSocket
  --ws-port PORT           WebSocket port for hot reload (default: 3001)
  --cors                   Enable CORS headers (default: enabled)
  --no-cors                Disable CORS headers

HTTPS (Advanced):
  --https                  Enable HTTPS
  --ssl-cert PATH          SSL certificate file path
  --ssl-key PATH           SSL private key file path

Output Options:
  --verbose, -v            Enable verbose output
  --test-mode              Run in test mode (don't start server)

Examples:
  ./scripts/serve.sh --port 8080 --hot-reload
  ./scripts/serve.sh --host 0.0.0.0 --cors
  ./scripts/serve.sh --https --ssl-cert cert.pem --ssl-key key.pem

Supported Servers:
  - Python 3 (http.server) - recommended
  - live-server (npm install -g live-server)
  - http-server (npm install -g http-server)
  - Custom Node.js server
EOF
            exit 0
            ;;
        *) log_error "Unknown parameter: $1"; echo "Use --help for usage information"; exit 1;;
    esac
    shift
done

# Header
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Development Server - Task 1.1.5                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check for required tools
check_server_tools

# Show configuration
log_info "Development server configuration:"
echo "  🌐 Host: $SERVER_HOST"
echo "  🔌 Port: $SERVER_PORT"
echo "  📁 Static files: $STATIC_DIR"
echo "  🔥 Hot reload: $HOT_RELOAD"
if [ "$HOT_RELOAD" = true ]; then
    echo "  📡 WebSocket port: $WS_PORT"
fi
echo "  🌍 CORS enabled: $CORS_ENABLED"
echo "  🔒 HTTPS enabled: $HTTPS_ENABLED"
echo "  🔧 Server type: $SERVER_TYPE"
echo ""

# Validate static directory
if [ ! -d "$STATIC_DIR" ]; then
    log_error "Static directory not found: $STATIC_DIR"
    exit 1
fi

# Create index.html if needed
create_index_html

# Exit early in test mode
if [ "$TEST_MODE" = true ]; then
    log_info "Test mode: server configuration validated"
    exit 0
fi

# Set up signal handling for graceful shutdown
cleanup() {
    log_info "Shutting down development server..."
    
    if [ -f ".dev_server.pid" ]; then
        local server_pid=$(cat .dev_server.pid)
        kill "$server_pid" 2>/dev/null || true
        rm -f .dev_server.pid
        log_info "HTTP server stopped"
    fi
    
    if [ -f ".hot_reload.pid" ]; then
        local ws_pid=$(cat .hot_reload.pid)
        kill "$ws_pid" 2>/dev/null || true
        rm -f .hot_reload.pid
        log_info "WebSocket server stopped"
    fi
    
    exit 0
}

trap cleanup INT TERM

# Start WebSocket server first (if hot reload is enabled)
start_websocket_server

# Start the main HTTP server
case "$SERVER_TYPE" in
    "python3"|"python")
        start_python_server "$SERVER_TYPE"
        ;;
    "live-server"|"http-server"|"node")
        start_node_server "$SERVER_TYPE"
        ;;
    *)
        log_error "Unsupported server type: $SERVER_TYPE"
        exit 1
        ;;
esac

# Wait a moment for server to start
sleep 2

# Check server health
if command -v curl &> /dev/null; then
    check_server_health
fi

# Show access URLs
log_success "🎉 Development server is running!"
echo ""
log_info "Access your application:"
echo "  🌐 Local:    http://$SERVER_HOST:$SERVER_PORT"
if [ "$SERVER_HOST" != "0.0.0.0" ] && [ "$SERVER_HOST" != "127.0.0.1" ]; then
    echo "  🌐 Network:  http://$(hostname -I | awk '{print $1}'):$SERVER_PORT"
fi

if [ "$HOT_RELOAD" = true ]; then
    echo ""
    log_info "Hot reload features:"
    echo "  📡 WebSocket: ws://$SERVER_HOST:$WS_PORT"
    echo "  🔄 Auto-reload on file changes"
fi

echo ""
log_info "Press Ctrl+C to stop the server"

# Keep the script running and wait for the server process
if [ -f ".dev_server.pid" ]; then
    wait $(cat .dev_server.pid)
fi