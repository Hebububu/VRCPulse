# Alert Policies

This directory defines the **Business Logic** and policies for detecting incidents and broadcasting alerts.

## Documents

*   **[policy-user-threshold.md](./policy-user-threshold.md)**: User-driven Alerts
    *   Report accumulation algorithms
    *   Threshold definitions for broadcasting
*   **[policy-vrchat-status.md](./policy-vrchat-status.md)**: Official API Integration
    *   VRChat Status API event detection
    *   Immediate broadcast conditions
*   **[policy-cloudfront.md](./policy-cloudfront.md)**: Metric-based Anomaly Detection
    *   Identifying patterns in CloudFront metrics
    *   Automatic alert triggers