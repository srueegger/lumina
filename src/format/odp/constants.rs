// ODP XML namespaces
pub const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
pub const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
pub const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
pub const NS_DRAW: &str = "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0";
pub const NS_PRESENTATION: &str = "urn:oasis:names:tc:opendocument:xmlns:presentation:1.0";
pub const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
pub const NS_SVG: &str = "urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0";
pub const NS_XLINK: &str = "http://www.w3.org/1999/xlink";
pub const NS_META: &str = "urn:oasis:names:tc:opendocument:xmlns:meta:1.0";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_MANIFEST: &str = "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0";

pub const ODP_MIMETYPE: &str = "application/vnd.oasis.opendocument.presentation";

/// Convert points to centimeters (ODP uses cm)
pub fn pt_to_cm(pt: f64) -> f64 {
    pt / 28.3465
}

/// Convert centimeters to points
pub fn cm_to_pt(cm: f64) -> f64 {
    cm * 28.3465
}

/// Format a dimension in cm for ODP
pub fn format_cm(pt: f64) -> String {
    format!("{:.4}cm", pt_to_cm(pt))
}

/// Parse a dimension string like "10.5cm" to points
pub fn parse_cm(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(val) = s.strip_suffix("cm") {
        val.parse::<f64>().ok().map(cm_to_pt)
    } else if let Some(val) = s.strip_suffix("in") {
        val.parse::<f64>().ok().map(|v| v * 72.0)
    } else if let Some(val) = s.strip_suffix("pt") {
        val.parse::<f64>().ok()
    } else {
        s.parse::<f64>().ok().map(cm_to_pt)
    }
}
