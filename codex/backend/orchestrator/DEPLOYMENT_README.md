# Codex Orchestrator Deployment Guide

This guide covers the complete setup for monitoring and deployment of the Codex Orchestrator service using Kubernetes and Prometheus.

## Prerequisites

- Kubernetes cluster (local or cloud)
- kubectl configured to access your cluster
- Docker Hub or container registry account
- GitHub repository for CI/CD

## Quick Start

### 1. Deploy Monitoring Stack

```bash
# Create namespaces
kubectl apply -f k8s/codex-namespace.yaml
kubectl apply -f k8s/monitoring-namespace.yaml

# Deploy Prometheus
kubectl apply -f k8s/prometheus-config.yaml
kubectl apply -f k8s/prometheus-deployment.yaml

# Deploy Grafana (optional)
kubectl apply -f k8s/grafana-deployment.yaml

# Deploy alert rules
kubectl apply -f k8s/alert-rules.yaml
```

### 2. Set Up GitHub Secrets

In your GitHub repository settings (Settings > Secrets > Actions), add:

1. `REGISTRY_USERNAME` - Your container registry username
2. `REGISTRY_PASSWORD` - Your container registry password/token
3. `KUBE_CONFIG` - Your Kubernetes config file (base64 encoded)

To encode your kubeconfig:
```bash
cat ~/.kube/config | base64 | pbcopy
```

### 3. Deploy the Application

```bash
# Apply the orchestrator deployment
kubectl apply -f k8s/deployment.yaml

# Verify deployment
kubectl get pods -n codex
kubectl get svc -n codex

# View logs
kubectl logs -f -n codex -l app=codex-orchestrator
```

## Access Services

- **Codex Orchestrator**: `http://localhost:3000`
- **Prometheus UI**: `http://localhost:30090`
- **Grafana** (optional): `http://localhost:30300` (admin/admin)

## Monitoring

### Verify Prometheus Targets

1. Access Prometheus at `http://localhost:30090`
2. Go to **Status > Targets** to verify the orchestrator service is being scraped
3. Check **Status > Configuration** for any configuration issues

### Query Metrics

Use the Graph tab in Prometheus to query metrics:
- `codex_requests_total` - Total number of requests
- `codex_request_duration_seconds` - Request duration histogram
- `codex_active_connections` - Current active connections

### Alert Rules

The following alerts are configured:
- **HighErrorRate**: Triggers when error rate exceeds 10% for 10 minutes

## Troubleshooting

### Check Pod Status
```bash
kubectl get pods -n codex -n monitoring
```

### View Pod Logs
```bash
# Orchestrator logs
kubectl logs -f -n codex -l app=codex-orchestrator

# Prometheus logs
kubectl logs -f -n monitoring -l app=prometheus
```

### Debug Services
```bash
# Check service endpoints
kubectl get endpoints -n codex -n monitoring

# Port forward for local access
kubectl port-forward -n codex svc/codex-orchestrator 3000:3000
```

## CI/CD Variables

For GitHub Actions, ensure these secrets are set:

1. `DOCKER_USERNAME` - Docker Hub username
2. `DOCKER_PASSWORD` - Docker Hub password/token
3. `KUBE_CONFIG` - Base64 encoded kubeconfig

## Configuration Files

- `k8s/codex-namespace.yaml` - Codex application namespace
- `k8s/monitoring-namespace.yaml` - Monitoring namespace
- `k8s/prometheus-config.yaml` - Prometheus scraping configuration
- `k8s/prometheus-deployment.yaml` - Prometheus deployment and service
- `k8s/alert-rules.yaml` - Alert rules for monitoring
- `k8s/grafana-deployment.yaml` - Grafana deployment (optional)
- `k8s/deployment.yaml` - Main orchestrator deployment (create separately)

## Metrics Endpoint

The orchestrator service exposes metrics at `/api/v1/metrics` endpoint on port 3000. Prometheus scrapes this endpoint every 5 seconds.

## Customizing Configuration

### Update Scrape Targets

Edit `k8s/prometheus-config.yaml` to modify scrape targets or add new services:

```yaml
scrape_configs:
  - job_name: 'your-service'
    scrape_interval: 5s
    static_configs:
      - targets: ['your-service.namespace.svc.cluster.local:8080']
```

### Add Custom Alert Rules

Edit `k8s/alert-rules.yaml` to add application-specific alerts:

```yaml
- alert: CustomAlert
  expr: your_metric > threshold
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Custom alert triggered"
```

## Security Considerations

- Use proper RBAC for service accounts
- Enable authentication on Prometheus and Grafana in production
- Use secrets management for sensitive configuration
- Implement network policies to restrict traffic

## Production Deployment

For production deployments, consider:

1. **Persistent Storage**: Use PersistentVolumeClaims instead of emptyDir
2. **Resource Limits**: Set proper resource requests and limits
3. **High Availability**: Deploy multiple Prometheus replicas
4. **Ingress**: Use Ingress controllers for external access
5. **Authentication**: Enable authentication on monitoring endpoints
6. **TLS**: Use TLS certificates for secure communication
