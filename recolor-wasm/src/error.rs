use thiserror::Error;

pub type Result<T> = std::result::Result<T, RecolorError>;

#[derive(Error, Debug)]
pub enum RecolorError {
    #[error("Missing input image")]
    MissingInputImage,
    #[error("Cannot open image")]
    IoError(#[from] std::io::Error),
    #[error("Cannot decode image")]
    DecodeError(#[from] image::error::ImageError),
    #[error("Cannot sample enough initial points for K-Means")]
    KMeansInitError,
    #[error("Cannot parse LAB line, expected <L> <a> <b>, got `{0}`")]
    LABLineError(String),
    #[error("Cannot perform Gauss Elimination on Blend Matrix")]
    GaussError,
}
