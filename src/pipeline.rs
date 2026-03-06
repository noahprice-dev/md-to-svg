pub mod pipeline {
    use std::{io, path::{Path, PathBuf}};

    use cosmic_text::FontSystem;
    use markdown::MdxSignal;

    use crate::{MdToSvgError, layout::SvgConfig};

    
    pub fn process_md_to_svg(
        input_path: &Path,
        font_system: &mut FontSystem,
        cfg: &SvgConfig,
    ) -> Result<String, MdToSvgError> {
        todo!()
    }
}
