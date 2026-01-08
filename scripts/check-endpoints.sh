#!/bin/bash
set -euo pipefail

# ==============================================================================
# VRCPulse - API Endpoint Verification Script
# ==============================================================================
#
# This script verifies the connectivity and data structure of all external APIs
# used by VRCPulse. It serves as a diagnostic tool and a living documentation
# of the API contracts we rely on.
#
# Usage: ./scripts/check-endpoints.sh
#
# Exit Codes:
#   0 - All tests passed
#   1 - Dependency missing or test failed
#
# ------------------------------------------------------------------------------
# 1. VRChat Status API (Atlassian Statuspage)
# ------------------------------------------------------------------------------
# Base URL: https://status.vrchat.com/api/v2
#
# VRCPulse relies on this API for:
# - Determining the overall health of VRChat servers.
# - Tracking specific component statuses (API, Website, etc.).
# - Monitoring incidents (outages) and scheduled maintenances.
#
# Key Endpoints:
# - summary.json:
#     [CRITICAL] The primary endpoint for our polling logic. It aggregates
#     status, components, and incidents into a single payload, allowing us
#     to minimize HTTP requests.
#
# - status.json:
#     Provides the high-level system indicator (e.g., "minor", "critical").
#     Used for quick health checks.
#
# - components.json:
#     Lists all system components with their individual status.
#     Used to populate the `component_logs` table for granular history.
#
# - incidents.json:
#     Contains the full history of past incidents.
#     Used to backfill the `incidents` and `incident_updates` tables.
#
# - incidents/unresolved.json:
#     Returns only currently active incidents.
#     Used for high-frequency checks to trigger immediate alerts.
#
# - scheduled-maintenances/{upcoming, active}.json:
#     Tracks planned downtimes to inform users via the `maintenances` table.
#
# ------------------------------------------------------------------------------
# 2. CloudFront Metrics API (Unofficial / Time-Series)
# ------------------------------------------------------------------------------
# Base URL: https://d31qqo63tn8lj0.cloudfront.net
#
# VRCPulse relies on this API for:
# - Generating visual dashboards with `plotters`.
# - Detecting anomalies in latency or user counts.
#
# Data Format: JSON Array `[[timestamp_unix, value], ...]`
#
# Key Endpoints:
# - apilatency.json: API response time (ms). Key metric for lag detection.
# - visits.json: Estimated concurrent user count/traffic.
# - apirequests.json: Total API request volume.
# - apierrors.json: API error rate.
# - extauth_*: Steam/Oculus specific authentication metrics.
#
# ==============================================================================

# ------------------------------------------------------------------------------
# Configuration
# ------------------------------------------------------------------------------
BASE_STATUS_URL="https://status.vrchat.com/api/v2"
BASE_METRIC_URL="https://d31qqo63tn8lj0.cloudfront.net"

PASS_COUNT=0
FAIL_COUNT=0

# ------------------------------------------------------------------------------
# Helper Functions
# ------------------------------------------------------------------------------
print_header() {
    echo -e "\n\033[1;36m>>> $1\033[0m"
}

print_subheader() {
    echo -e "\033[1;33m[TEST] $1\033[0m"
}

print_pass() {
    echo -e "\033[1;32m[PASS]\033[0m $1"
    PASS_COUNT=$((PASS_COUNT + 1))
}

print_fail() {
    echo -e "\033[1;31m[FAIL]\033[0m $1"
    FAIL_COUNT=$((FAIL_COUNT + 1))
}

print_error() {
    echo -e "\033[1;31m[ERROR]\033[0m $1" >&2
}

# Check if required commands exist
check_dependencies() {
    local missing=()

    for cmd in curl jq; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        print_error "Missing required dependencies: ${missing[*]}"
        print_error "Please install them before running this script."
        exit 1
    fi
}

# Fetch URL and validate HTTP status + JSON structure
# Usage: fetch_and_validate <url> <jq_validation_expr> <description>
# Returns: 0 on success, 1 on failure
fetch_and_validate() {
    local url="$1"
    local jq_expr="$2"
    local description="$3"

    local tmp_file
    tmp_file=$(mktemp)
    trap "rm -f '$tmp_file'" RETURN

    local http_code
    http_code=$(curl -sS -w '%{http_code}' -o "$tmp_file" "$url" 2>&1) || {
        print_fail "$description - curl failed: $http_code"
        return 1
    }

    if [[ "$http_code" != "200" ]]; then
        print_fail "$description - HTTP $http_code"
        return 1
    fi

    if ! jq -e "$jq_expr" "$tmp_file" &>/dev/null; then
        print_fail "$description - JSON structure validation failed"
        echo "  Expected: $jq_expr"
        return 1
    fi

    return 0
}

# Validate VRChat Status API endpoint
# Usage: validate_status_endpoint <endpoint> <jq_expr> <description>
validate_status_endpoint() {
    local endpoint="$1"
    local jq_expr="$2"
    local description="$3"

    print_subheader "GET $endpoint"

    if fetch_and_validate "${BASE_STATUS_URL}${endpoint}" "$jq_expr" "$description"; then
        print_pass "$description"
        return 0
    fi
    return 1
}

# Validate CloudFront Metrics API endpoint
# Usage: validate_metric_endpoint <metric_name>
validate_metric_endpoint() {
    local metric="$1"
    local url="${BASE_METRIC_URL}/${metric}.json"

    print_subheader "GET /${metric}.json"

    # CloudFront metrics are arrays of [timestamp, value] pairs
    local jq_expr='type == "array" and (length == 0 or (.[0] | type == "array" and length == 2))'

    if fetch_and_validate "$url" "$jq_expr" "$metric"; then
        print_pass "$metric - valid time-series array"
        return 0
    fi
    return 1
}

# ------------------------------------------------------------------------------
# Main Execution
# ------------------------------------------------------------------------------
main() {
    print_header "Checking dependencies..."
    check_dependencies
    echo "All dependencies found: curl, jq"

    # --------------------------------------------------------------------------
    # VRChat Status API Tests
    # --------------------------------------------------------------------------
    print_header "Testing VRChat Status API ($BASE_STATUS_URL)"

    # 1. Summary (Main Data Source) - Most critical endpoint
    # Expected: { page, status, components[], incidents[], scheduled_maintenances[] }
    validate_status_endpoint "/summary.json" \
        'has("page", "status", "components", "incidents", "scheduled_maintenances")' \
        "summary.json (Primary Polling Source)"

    # 2. Status
    # Expected: { page, status: { indicator, description } }
    validate_status_endpoint "/status.json" \
        'has("page", "status") and (.status | has("indicator", "description"))' \
        "status.json"

    # 3. Components
    # Expected: { page, components[] }
    validate_status_endpoint "/components.json" \
        'has("page", "components") and (.components | type == "array") and (.components | length > 0)' \
        "components.json"

    # 4. Incidents (Unresolved)
    # Expected: { page, incidents[] }
    validate_status_endpoint "/incidents/unresolved.json" \
        'has("page", "incidents") and (.incidents | type == "array")' \
        "incidents/unresolved.json (Active Alerts)"

    # 5. Scheduled Maintenances
    # Expected: { page, scheduled_maintenances[] }
    validate_status_endpoint "/scheduled-maintenances/upcoming.json" \
        'has("page", "scheduled_maintenances") and (.scheduled_maintenances | type == "array")' \
        "scheduled-maintenances/upcoming.json"

    # --------------------------------------------------------------------------
    # CloudFront Metrics API Tests
    # --------------------------------------------------------------------------
    print_header "Testing CloudFront Metrics API ($BASE_METRIC_URL)"

    local metrics=(
        "apilatency"
        "visits"
        "apirequests"
        "apierrors"
        "extauth_steam"
        "extauth_steam_count"
        "extauth_oculus"
        "extauth_oculus_count"
    )

    for metric in "${metrics[@]}"; do
        validate_metric_endpoint "$metric"
    done

    # --------------------------------------------------------------------------
    # Summary
    # --------------------------------------------------------------------------
    print_header "Test Summary"
    echo -e "  Passed: \033[1;32m${PASS_COUNT}\033[0m"
    echo -e "  Failed: \033[1;31m${FAIL_COUNT}\033[0m"

    if [[ $FAIL_COUNT -gt 0 ]]; then
        print_error "Some tests failed. Check the output above for details."
        exit 1
    fi

    echo -e "\n\033[1;32mAll tests completed successfully.\033[0m"
    exit 0
}

main "$@"
