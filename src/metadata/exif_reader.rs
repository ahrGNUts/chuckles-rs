use std::path::Path;

/// Parsed EXIF camera data for display in the metadata panel.
#[derive(Debug, Clone, Default)]
pub struct ExifData {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<String>,
    pub focal_length: Option<String>,
    pub date_taken: Option<String>,
    pub gps_latitude: Option<String>,
    pub gps_longitude: Option<String>,
    /// All raw tags as (name, value) pairs.
    pub raw_tags: Vec<(String, String)>,
}

impl ExifData {
    pub fn has_camera_data(&self) -> bool {
        self.camera_make.is_some()
            || self.camera_model.is_some()
            || self.aperture.is_some()
            || self.shutter_speed.is_some()
            || self.iso.is_some()
    }
}

/// Read EXIF data from an image file. Returns None if the file has no EXIF data
/// or if reading fails.
pub fn read_exif(path: &Path) -> Option<ExifData> {
    let file = std::fs::File::open(path).ok()?;
    let mut buf_reader = std::io::BufReader::new(file);
    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut buf_reader).ok()?;

    let mut data = ExifData::default();

    for field in exif.fields() {
        let tag_name = field.tag.to_string();
        let value = field.display_value().with_unit(&exif).to_string();

        match field.tag {
            exif::Tag::Make => data.camera_make = Some(value.clone()),
            exif::Tag::Model => data.camera_model = Some(value.clone()),
            exif::Tag::LensModel => data.lens = Some(value.clone()),
            exif::Tag::FNumber => data.aperture = Some(format!("f/{value}")),
            exif::Tag::ExposureTime => data.shutter_speed = Some(value.clone()),
            exif::Tag::PhotographicSensitivity => data.iso = Some(value.clone()),
            exif::Tag::FocalLength => data.focal_length = Some(value.clone()),
            exif::Tag::DateTimeOriginal => data.date_taken = Some(value.clone()),
            exif::Tag::GPSLatitude => data.gps_latitude = Some(value.clone()),
            exif::Tag::GPSLongitude => data.gps_longitude = Some(value.clone()),
            _ => {}
        }

        data.raw_tags.push((tag_name, value));
    }

    Some(data)
}
