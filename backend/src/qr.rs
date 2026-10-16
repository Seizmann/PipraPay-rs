use qrcode::QrCode;

/// Render a QR code as a self-contained SVG string (server-side, per plan —
/// no frontend QR dependency).
pub fn svg(data: &str) -> Result<String, anyhow::Error> {
    let code = QrCode::new(data.as_bytes())?;
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .dark_color(qrcode::render::svg::Color("#000000"))
        .light_color(qrcode::render::svg::Color("#ffffff"))
        .build();
    Ok(svg)
}

#[cfg(test)]
mod tests {
    #[test]
    fn svg_output_shape() {
        let svg = super::svg("https://example.com/otpauth?x=1").unwrap();
        assert!(
            svg.starts_with("<?xml") || svg.starts_with("<svg"),
            "{}",
            &svg[..60.min(svg.len())]
        );
        assert!(svg.contains("svg"));
    }
}
