//! Dashboard generation
//!
//! Generates a 6-chart dashboard PNG image.

use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::series::{AreaSeries, LineSeries};
use plotters::style::{Color, IntoFont, RGBColor};
use sea_orm::DatabaseConnection;

use crate::visualization::query::{MetricData, load_metric_as_percent, load_metric_downsampled};
use crate::visualization::theme::*;

/// Y-axis format for charts
#[derive(Clone, Copy)]
pub enum YAxisFormat {
    /// Count format: 0, 50k, 100k
    Count,
    /// Fixed percentage: 0%, 50%, 100%
    Percent,
    /// Dynamic percentage: 0% to max+10%
    PercentAuto,
    /// No Y-axis labels
    Hidden,
}

/// Dashboard statistics for embed fields
#[derive(Debug, Clone)]
pub struct DashboardStats {
    pub online_users_avg: f64,
    pub online_users_max: f64,
    pub api_error_rate_avg: f64,
    pub steam_success_avg: f64,
    pub meta_success_avg: f64,
}

/// Generate dashboard PNG and return bytes with stats
pub async fn generate_dashboard(
    db: &DatabaseConnection,
) -> Result<(Vec<u8>, DashboardStats), Box<dyn std::error::Error + Send + Sync>> {
    // Load all 6 metrics
    let online_users = load_metric_downsampled(db, "visits").await?;
    let api_latency = load_metric_downsampled(db, "api_latency").await?;
    let api_requests = load_metric_downsampled(db, "api_requests").await?;
    let api_error_rate = load_metric_as_percent(db, "api_errors").await?;
    let steam_success = load_metric_as_percent(db, "extauth_steam").await?;
    let meta_success = load_metric_as_percent(db, "extauth_oculus").await?;

    // Calculate stats
    let stats = DashboardStats {
        online_users_avg: online_users.avg(),
        online_users_max: online_users.max(),
        api_error_rate_avg: api_error_rate.avg(),
        steam_success_avg: steam_success.avg(),
        meta_success_avg: meta_success.avg(),
    };

    // Generate PNG in memory
    let mut buffer = vec![0u8; (IMAGE_SIZE * IMAGE_SIZE * 3) as usize];

    {
        let root =
            BitMapBackend::with_buffer(&mut buffer, (IMAGE_SIZE, IMAGE_SIZE)).into_drawing_area();
        root.fill(&BG_COLOR)?;

        // Split into grid: 3 rows x 2 cols
        let areas = root.margin(30, 30, 30, 30).split_evenly((3, 2));

        // Row 1: Online Users, API Latency
        draw_chart(
            &areas[0],
            "Online Users",
            &online_users,
            GRAPH_COLOR,
            YAxisFormat::Count,
        )?;
        draw_chart(
            &areas[1],
            "API Latency",
            &api_latency,
            GRAPH_COLOR,
            YAxisFormat::Hidden,
        )?;

        // Row 2: API Requests, API Error Rate
        draw_chart(
            &areas[2],
            "API Requests",
            &api_requests,
            GRAPH_COLOR,
            YAxisFormat::Hidden,
        )?;
        draw_chart(
            &areas[3],
            "API Error Rate",
            &api_error_rate,
            RED,
            YAxisFormat::PercentAuto,
        )?;

        // Row 3: Steam Auth Success Rate, Meta Auth Success Rate
        draw_chart(
            &areas[4],
            "Steam Auth Success Rate",
            &steam_success,
            GREEN,
            YAxisFormat::Percent,
        )?;
        draw_chart(
            &areas[5],
            "Meta Auth Success Rate",
            &meta_success,
            GREEN,
            YAxisFormat::Percent,
        )?;

        root.present()?;
    }

    // Encode to PNG
    let png_bytes = encode_png(&buffer, IMAGE_SIZE, IMAGE_SIZE)?;

    Ok((png_bytes, stats))
}

/// Draw a single chart
fn draw_chart(
    area: &plotters::drawing::DrawingArea<BitMapBackend, plotters::coord::Shift>,
    title: &str,
    data: &MetricData,
    color: RGBColor,
    y_format: YAxisFormat,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if data.is_empty() {
        return Ok(());
    }

    let max_val = data.max();
    let y_max =
        match y_format {
            YAxisFormat::Percent => 100.0,
            YAxisFormat::PercentAuto | YAxisFormat::Count | YAxisFormat::Hidden => {
                if max_val == 0.0 { 1.0 } else { max_val * 1.1 }
            }
        };

    let start_time = data.timestamps.first().unwrap();
    let end_time = data.timestamps.last().unwrap();

    let mut chart = ChartBuilder::on(area)
        .caption(
            title,
            ("sans-serif", TITLE_FONT_SIZE)
                .into_font()
                .color(&TEXT_COLOR),
        )
        .margin(20)
        .x_label_area_size(70)
        .y_label_area_size(120)
        .build_cartesian_2d(0..data.values.len(), 0.0..y_max)?;

    chart
        .configure_mesh()
        .x_labels(5)
        .y_labels(5)
        .x_label_formatter(&|x| {
            if *x == 0 {
                start_time.format("%H:%M").to_string()
            } else if *x >= data.values.len() - 1 {
                end_time.format("%H:%M").to_string()
            } else if *x < data.timestamps.len() {
                data.timestamps[*x].format("%H:%M").to_string()
            } else {
                String::new()
            }
        })
        .y_label_formatter(&move |y| match y_format {
            YAxisFormat::Count => {
                if *y >= 1000.0 {
                    format!("{:.0}k", y / 1000.0)
                } else {
                    format!("{:.0}", y)
                }
            }
            YAxisFormat::Percent => format!("{:.0}%", y),
            YAxisFormat::PercentAuto => {
                if *y >= 1.0 {
                    format!("{:.1}%", y)
                } else if *y >= 0.01 {
                    format!("{:.2}%", y)
                } else {
                    format!("{:.4}%", y)
                }
            }
            YAxisFormat::Hidden => String::new(),
        })
        .x_label_style(
            ("sans-serif", LABEL_FONT_SIZE)
                .into_font()
                .color(&MUTED_COLOR),
        )
        .y_label_style(
            ("sans-serif", LABEL_FONT_SIZE)
                .into_font()
                .color(&MUTED_COLOR),
        )
        .axis_style(MUTED_COLOR)
        .bold_line_style(MUTED_COLOR.mix(0.2))
        .light_line_style(MUTED_COLOR.mix(0.1))
        .draw()?;

    // Draw area
    chart.draw_series(AreaSeries::new(
        data.values.iter().enumerate().map(|(i, v)| (i, *v)),
        0.0,
        color.mix(0.3),
    ))?;

    // Draw line
    chart.draw_series(LineSeries::new(
        data.values.iter().enumerate().map(|(i, v)| (i, *v)),
        color.stroke_width(4),
    ))?;

    Ok(())
}

/// Encode raw RGB buffer to PNG
fn encode_png(
    buffer: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Cursor;

    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(Cursor::new(&mut png_data), width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(buffer)?;
    }
    Ok(png_data)
}
