# System Architecture

This directory covers the **Internal Systems**, data structures, and the visualization engine.

## Documents

*   **[database-schema.md](./database-schema.md)**: Database Design
    *   SQLite table structures (ERD)
    *   Sea-ORM Entity definitions
*   **[data-collector.md](./data-collector.md)**: Data Collector
    *   VRChat Status API & CloudFront API integration
    *   Cron Job scheduling and polling logic
*   **[visualization-engine.md](./visualization-engine.md)**: Visualization Engine
    *   `plotters` implementation details
    *   Time-series data rendering and buffer handling