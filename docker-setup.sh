#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Configuration
WORKSPACE_DIR="./workspace"
REQUIRED_DOCKER_VERSION="24.0.0"
REQUIRED_COMPOSE_VERSION="2.38.0"

# Helper functions
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_header() {
    echo ""
    echo "========================================"
    echo "$1"
    echo "========================================"
    echo ""
}

# Version comparison function
version_ge() {
    printf '%s\n%s\n' "$2" "$1" | sort -V -C
}

# Check if Docker is installed
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed."
        echo "Please install Docker Desktop from: https://docs.docker.com/get-docker/"
        exit 1
    fi

    local docker_version=$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo "0.0.0")

    if ! version_ge "$docker_version" "$REQUIRED_DOCKER_VERSION"; then
        print_warning "Docker version $docker_version detected. Version $REQUIRED_DOCKER_VERSION or later recommended."
    else
        print_success "Docker is installed (version $docker_version)"
    fi
}

# Check if Docker Compose is installed
check_docker_compose() {
    if ! docker compose version &> /dev/null; then
        print_error "Docker Compose V2 is not installed."
        echo "Please update Docker Desktop or install Docker Compose V2"
        echo "Visit: https://docs.docker.com/compose/install/"
        exit 1
    fi

    local compose_version=$(docker compose version --short 2>/dev/null || echo "0.0.0")
    compose_version=${compose_version#v} # Remove 'v' prefix if present

    if ! version_ge "$compose_version" "$REQUIRED_COMPOSE_VERSION"; then
        print_error "Docker Compose version $compose_version detected."
        echo "Docker Compose $REQUIRED_COMPOSE_VERSION or later is required for AI models support."
        echo "Please update Docker Desktop to the latest version."
        exit 1
    fi

    print_success "Docker Compose is installed (version $compose_version)"
}

# Check if Docker Desktop has AI features
check_docker_ai() {
    print_info "Checking Docker AI features..."

    # Try to validate the compose file with models section
    local config_output=$(docker compose config 2>&1)
    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        # Check if models section is present in output
        if echo "$config_output" | grep -q "^models:"; then
            print_success "Docker AI features are available"
            return 0
        fi
    fi

    # Check if error is specifically about models not being supported
    if echo "$config_output" | grep -qi "models.*not.*supported\|models.*invalid\|models.*unknown"; then
        print_warning "Docker AI model runner may not be available"
        echo ""
        echo "Docker AI features may not be enabled. To enable:"
        echo "1. Open Docker Desktop"
        echo "2. Go to Settings → Features in Development"
        echo "3. Enable 'Docker AI' features"
        echo "4. Restart Docker Desktop"
        echo ""
        echo "Or continue anyway if you have Docker AI already enabled."
        echo ""
        read -p "Continue with setup? (Y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            print_info "Setup cancelled"
            exit 0
        fi
    else
        # Configuration is valid (might have warnings but no errors)
        print_success "Docker Compose configuration is valid"
        return 0
    fi
}

# Check if Docker daemon is running
check_docker_daemon() {
    if ! docker info &> /dev/null; then
        print_error "Docker daemon is not running"
        echo "Please start Docker Desktop and try again"
        exit 1
    fi
    print_success "Docker daemon is running"
}

# Check available disk space
check_disk_space() {
    # macOS compatible disk space check
    local available_gb=$(df -h "$SCRIPT_DIR" | awk 'NR==2 {print $4}' | sed 's/[^0-9.]//g' | cut -d'.' -f1)

    if [ -z "$available_gb" ]; then
        print_info "Could not determine disk space"
        return 0
    fi

    if [ "$available_gb" -lt 10 ]; then
        print_warning "Low disk space: ${available_gb}GB available"
        echo "At least 10GB recommended for model and container images"
    else
        print_success "Sufficient disk space available (${available_gb}GB)"
    fi
}

# Setup workspace
setup_workspace() {
    print_header "Setting Up Workspace"

    if [ ! -d "$WORKSPACE_DIR" ]; then
        mkdir -p "$WORKSPACE_DIR"
        print_success "Created workspace directory"
    else
        print_success "Workspace directory already exists"
    fi

    # Set proper permissions
    chmod 755 "$WORKSPACE_DIR"

    # Create a sample README if empty
    if [ -z "$(ls -A $WORKSPACE_DIR)" ]; then
        cat > "$WORKSPACE_DIR/README.md" << 'EOF'
# Squid Workspace

Place your code files here for analysis with Squid.

## Usage

Files in this directory are accessible to Squid at `/data/workspace/` inside the container.

Example:
```bash
docker compose exec squid /app/squid review --file /data/workspace/myproject/src/main.rs
```
EOF
        print_info "Created sample README in workspace"
    fi
}

# Build Docker images
build_images() {
    print_header "Building Docker Images"

    print_info "Building Squid server image..."
    if docker compose build; then
        print_success "Docker images built successfully"
    else
        print_error "Failed to build Docker images"
        exit 1
    fi
}

# Pull model
pull_model() {
    print_header "Pulling AI Models"

    print_info "Pulling models from Hugging Face..."
    echo "  • LLM: bartowski/qwen2.5-coder-7b-instruct (Q4_K_M) - ~4GB"
    echo "  • Embedding: nomic-ai/nomic-embed-text-v1.5 - ~270MB"
    print_warning "This may take several minutes depending on your internet connection"

    if docker compose pull; then
        print_success "Models pulled successfully"
    else
        print_error "Failed to pull models"
        echo "This may be due to:"
        echo "  • Docker AI features not enabled"
        echo "  • Network connectivity issues"
        echo "  • Hugging Face availability"
        exit 1
    fi
}

# Start services
start_services() {
    print_header "Starting Services"

    print_info "Starting containers..."
    if docker compose up -d; then
        print_success "Containers started"
    else
        print_error "Failed to start containers"
        exit 1
    fi

    print_info "Waiting for services to be ready..."

    # Wait for squid server to be healthy
    local max_wait=60
    local wait_time=0

    while [ $wait_time -lt $max_wait ]; do
        if docker compose ps squid 2>/dev/null | grep -q "Up"; then
            sleep 2
            if curl -sf http://localhost:3000/health > /dev/null 2>&1 || \
               docker compose ps squid | grep -q "healthy"; then
                print_success "Squid server is ready"
                return 0
            fi
        fi

        echo -n "."
        sleep 2
        wait_time=$((wait_time + 2))
    done

    echo ""
    print_warning "Service health check timed out, but containers may still be starting"
    print_info "Check logs with: docker compose logs -f"
}

# Show status
show_status() {
    print_header "Service Status"

    docker compose ps

    echo ""
    print_info "Access Points:"
    echo "  • Squid Web UI:  http://localhost:3000"
    echo ""

    print_info "Useful Commands:"
    echo "  • View logs:     docker compose logs -f"
    echo "  • Stop services: docker compose stop"
    echo "  • Start again:   docker compose start"
    echo "  • Restart:       docker compose restart"
    echo ""

    # Show environment variables
    print_info "Model Configuration:"
    docker compose exec -T squid env 2>/dev/null | grep "SQUID_" || true
}

# Stop services
stop_services() {
    print_header "Stopping Services"

    if docker compose stop; then
        print_success "Services stopped"
    else
        print_error "Failed to stop services"
        exit 1
    fi
}

# Restart services
restart_services() {
    print_header "Restarting Services"

    if docker compose restart; then
        print_success "Services restarted"
    else
        print_error "Failed to restart services"
        exit 1
    fi
}

# Show logs
show_logs() {
    print_info "Showing logs (Ctrl+C to exit)..."
    docker compose logs -f
}

# Test the setup
test_setup() {
    print_header "Testing Setup"

    print_info "Checking if services are running..."
    if ! docker compose ps squid | grep -q "Up"; then
        print_error "Squid service is not running"
        echo "Start it with: docker compose up -d"
        exit 1
    fi
    print_success "Squid service is running"

    print_info "Testing health endpoint..."
    if curl -sf http://localhost:3000/health > /dev/null 2>&1; then
        print_success "Health endpoint responding"
    else
        print_warning "Health endpoint not responding yet"
    fi

    print_info "Checking environment variables..."
    if docker compose exec -T squid env | grep -q "SQUID_URL"; then
        print_success "Environment variables configured"
        docker compose exec -T squid env | grep "SQUID_"
    else
        print_warning "Environment variables not found"
    fi

    echo ""
    print_success "Setup test complete"
}

# Clean everything
clean_all() {
    print_header "Cleaning Up"

    read -p "This will remove all containers and volumes. Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Cleanup cancelled"
        return 0
    fi

    print_info "Stopping and removing containers..."
    docker compose down -v

    print_success "Containers and volumes removed"

    read -p "Remove workspace directory? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$WORKSPACE_DIR"
        print_success "Workspace removed"
    fi

    print_info "Docker images are retained. Remove manually with:"
    echo "  docker compose down --rmi all"
}

# Update services
update_services() {
    print_header "Updating Services"

    print_info "Pulling latest images and models..."
    docker compose pull

    print_info "Rebuilding Squid server..."
    docker compose build --pull

    print_info "Restarting services..."
    docker compose up -d

    print_success "Update complete - both LLM and embedding models updated"
}

# Full setup
full_setup() {
    print_header "Squid Docker Setup"
    echo "Setting up Squid with Docker AI model runner"
    echo ""

    check_docker
    check_docker_daemon
    check_docker_compose
    check_disk_space
    check_docker_ai

    setup_workspace

    print_info "This will now:"
    echo "  1. Build the Squid server image"
    echo "  2. Pull the qwen2.5-coder LLM model (~4GB download)"
    echo "  3. Pull the nomic-embed-text embedding model (~270MB download)"
    echo "  4. Start all services with RAG enabled"
    echo ""
    read -p "Continue? (Y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        print_info "Setup cancelled"
        exit 0
    fi

    build_images
    pull_model
    start_services
    show_status

    print_header "Setup Complete! 🎉"
    print_success "Squid is now running with:"
    echo "  • LLM: bartowski/qwen2.5-coder-7b-instruct (Q4_K_M)"
    echo "  • Embedding: nomic-ai/nomic-embed-text-v1.5"
    echo "  • RAG: Enabled for semantic search"
    echo ""
    print_info "Next Steps:"
    echo "  1. Open http://localhost:3000 in your browser"
    echo "  2. Place your code in the ./workspace directory"
    echo "  3. Try: docker compose exec squid /app/squid ask 'How do I use Rust?'"
    echo ""
    print_info "Useful Commands:"
    echo "  • View logs:    ./docker-setup.sh logs"
    echo "  • Stop:         ./docker-setup.sh stop"
    echo "  • Restart:      ./docker-setup.sh restart"
    echo "  • Status:       ./docker-setup.sh status"
    echo ""
}

# Show usage
show_usage() {
    cat << EOF
Squid Docker Setup Script

Usage: $0 [COMMAND]

Commands:
  setup       Full setup (build + pull + start)
  build       Build Docker images only
  pull        Pull AI model only
  start       Start services
  stop        Stop services
  restart     Restart services
  status      Show service status
  logs        Show logs (live)
  test        Test the setup
  update      Update services and model
  clean       Remove everything
  help        Show this help message

Run without arguments for interactive menu.

Examples:
  $0 setup          # Complete setup
  $0 logs           # View logs
  $0 status         # Check status
  $0 clean          # Remove all

For more information, see docs/DOCKER.md
EOF
}

# Main menu
show_menu() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║   Squid Docker AI Setup Script        ║"
    echo "╚════════════════════════════════════════╝"
    echo ""
    echo "1)  Full setup (build + pull + start)"
    echo "2)  Build images"
    echo "3)  Pull model"
    echo "4)  Start services"
    echo "5)  Stop services"
    echo "6)  Restart services"
    echo "7)  Show status"
    echo "8)  Show logs"
    echo "9)  Test setup"
    echo "10) Update services"
    echo "11) Clean all"
    echo "0)  Exit"
    echo ""
    read -p "Select option: " choice

    case $choice in
        1) full_setup ;;
        2) build_images ;;
        3) pull_model ;;
        4) start_services; show_status ;;
        5) stop_services ;;
        6) restart_services ;;
        7) show_status ;;
        8) show_logs ;;
        9) test_setup ;;
        10) update_services ;;
        11) clean_all ;;
        0) exit 0 ;;
        *) print_error "Invalid option" ;;
    esac
}

# Handle command line arguments
if [ $# -eq 0 ]; then
    show_menu
else
    case "$1" in
        setup|init) full_setup ;;
        build) build_images ;;
        pull) pull_model ;;
        start) start_services; show_status ;;
        stop) stop_services ;;
        restart) restart_services ;;
        status) show_status ;;
        logs) show_logs ;;
        test) test_setup ;;
        update) update_services ;;
        clean) clean_all ;;
        help|--help|-h) show_usage ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            show_usage
            exit 1
            ;;
    esac
fi
