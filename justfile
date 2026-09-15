scraper_tag := "checkmk-kubernetes-agent-node-scraper:local"
scraper_target := "node-scraper-dev"

aggregator_tag := "checkmk-kubernetes-agent-cluster-aggregator:local"
aggregator_target := "cluster-aggregator-dev"

push := ""
push_ott := ""
push_url := ""
site_ca := ""
cluster_host_name := ""

# Build an image for Kubernetes using Docker
dockerize:
    docker build -t {{scraper_tag}} --target {{scraper_target}} -f docker/Dockerfile .
    docker build -t {{aggregator_tag}} --target {{aggregator_target}} -f docker/Dockerfile .

# Create a kind cluster for development
kind-create:
    sed "s#\\\$SRC_DIR\\\$#$(pwd)#" devel/kind-config.yaml | \
      kind create cluster --name checkmk-kubernetes-agent --config -

# Load images into the kind cluster, creating it if it does not exist
kind-load:
    kind load docker-image {{scraper_tag}} --name checkmk-kubernetes-agent
    kind load docker-image {{aggregator_tag}} --name checkmk-kubernetes-agent

# Load the helm chart into the kind cluster with devel/values.yaml
kind-helm-install: kind-registration-secret
    helm upgrade --install checkmk-agent ./charts/checkmk-agent \
      -n checkmk-monitoring --create-namespace -f devel/values.yaml \
      {{ if path_exists("devel/custom_values.yaml") == "true" { "-f devel/custom_values.yaml" } else { "" } }} \
      {{ if push != "" { "--set push.enabled=" + push } else { "" } }} \
      {{ if push_ott + site_ca != "" { "--set push.registrationSecret=checkmk-agent-push-registration" } else { "" } }} \
      {{ if push_url != "" { "--set push.url=" + push_url } else { "" } }} \
      {{ if site_ca != "" { "--set push.insecureSkipSiteCaVerification=false" } else { "" } }} \
      {{ if cluster_host_name != "" { "--set clusterHostName=" + cluster_host_name } else { "" } }}

# Create the dev registration Secret from the supplied credentials.
[private]
kind-registration-secret:
    {{ if push_ott + site_ca != "" { "kubectl create namespace checkmk-monitoring --dry-run=client -o yaml | kubectl apply -f -" } else { "true" } }}
    @{{ if push_ott + site_ca != "" { \
        "kubectl create secret generic checkmk-agent-push-registration -n checkmk-monitoring " + \
        (if push_ott != "" { "--from-literal=" + quote("token=" + push_ott) + " " } else { "" }) + \
        (if site_ca != "" { "--from-file=" + quote("site-ca-pem=" + site_ca) + " " } else { "" }) + \
        "--dry-run=client -o yaml | kubectl apply -f -" \
    } else { "true" } }}

# Delete the helm deployment from the kind cluster
kind-helm-delete:
    helm delete checkmk-agent -n checkmk-monitoring

# DEV ENV: Deploy the agent in Kind with source mounted at /src
kind-dev: dockerize kind-create kind-load kind-helm-install

# Remove the Kind dev cluster
kind-dev-teardown:
    kind delete cluster --name checkmk-kubernetes-agent

# Run kubeconform and ct lint over the helm chart
lint-helm:
    #!/usr/bin/env bash
    set -euo pipefail
    for values in charts/checkmk-agent/ci/*-values.yaml; do
      helm template myrelease charts/checkmk-agent -f "$values" | kubeconform -strict -summary
    done
    ct lint --all

# Run a rough subset of what runs in CI
sanity: lint-helm
    cargo doc --workspace --no-deps --document-private-items
    cargo test
    cargo clippy
    cargo fmt --check
