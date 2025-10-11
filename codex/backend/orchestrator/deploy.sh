#!/bin/bash

# Codex Orchestrator Deployment Script
# This script deploys the monitoring stack and orchestrator service

set -e

echo "🚀 Starting Codex Orchestrator deployment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    print_error "kubectl is not installed or not in PATH"
    exit 1
fi

# Check if cluster is accessible
if ! kubectl cluster-info &> /dev/null; then
    print_error "Cannot connect to Kubernetes cluster"
    exit 1
fi

print_status "Connected to Kubernetes cluster"

# Create codex namespace
print_status "Creating codex namespace..."
kubectl apply -f k8s/codex-namespace.yaml

# Deploy Prometheus
print_status "Deploying Prometheus..."
kubectl apply -f k8s/prometheus-config.yaml
kubectl apply -f k8s/prometheus-deployment.yaml

# Wait for Prometheus to be ready
print_status "Waiting for Prometheus to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/prometheus -n monitoring

# Deploy alert rules
print_status "Deploying alert rules..."
kubectl apply -f k8s/alert-rules.yaml

# Deploy Grafana (optional)
read -p "Deploy Grafana for visualization? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Deploying Grafana..."
    kubectl apply -f k8s/grafana-deployment.yaml

    print_status "Waiting for Grafana to be ready..."
    kubectl wait --for=condition=available --timeout=300s deployment/grafana -n monitoring

    print_status "Grafana will be available at http://localhost:30300 (admin/admin)"
fi

# Check if deployment.yaml exists
if [ ! -f "k8s/deployment.yaml" ]; then
    print_warning "k8s/deployment.yaml not found. Please create the main orchestrator deployment file."
    print_status "Run 'kubectl apply -f k8s/deployment.yaml' once you have created the deployment file."
else
    # Deploy the orchestrator
    print_status "Deploying Codex Orchestrator..."
    kubectl apply -f k8s/deployment.yaml

    print_status "Waiting for orchestrator pods to be ready..."
    kubectl wait --for=condition=available --timeout=300s deployment/codex-orchestrator -n codex
fi

# Show status
print_status "Deployment completed! Current status:"
echo
kubectl get pods -n codex -n monitoring --no-headers | awk '{print "  " $1 ": " $3}'
echo
kubectl get svc -n codex -n monitoring --no-headers | awk '{print "  " $1 ": " $3 ":" $5}'
echo

print_status "Access your services:"
if kubectl get svc codex-orchestrator -n codex &> /dev/null; then
    print_status "  - Codex Orchestrator: http://localhost:3000"
fi
print_status "  - Prometheus: http://localhost:30090"
if kubectl get svc grafana -n monitoring &> /dev/null; then
    print_status "  - Grafana: http://localhost:30300 (admin/admin)"
fi

print_status "For logs, run:"
print_status "  kubectl logs -f -n codex -l app=codex-orchestrator"

print_status "For monitoring setup, run:"
print_status "  kubectl logs -f -n monitoring -l app=prometheus"
