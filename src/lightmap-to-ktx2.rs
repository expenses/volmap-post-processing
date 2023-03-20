use half::f16;
use ktx2_tools::ktx2;
use std::borrow::Cow;

fn normalize_float(base: f32, mut float: f32) -> u8 {
    float /= base;
    if float.is_nan() {
        float = 0.0;
    }
    float = float.max(-1.0).min(1.0);

    float = (float + 1.0) * 128.0;
    float as u8
}

fn main() {
    let mut args = std::env::args().skip(1);

    let filename = args.next().unwrap();

    let base_image = image::open(&filename).unwrap();

    let sh_0 = base_image
        .crop_imm(0, 0, base_image.width() / 4, base_image.height())
        .to_rgba32f();

    let sh_1_x = base_image
        .crop_imm(
            base_image.width() / 4,
            0,
            base_image.width() / 4,
            base_image.height(),
        )
        .to_rgba32f();

    let sh_1_y = base_image
        .crop_imm(
            2 * base_image.width() / 4,
            0,
            base_image.width() / 4,
            base_image.height(),
        )
        .to_rgba32f();

    let sh_1_z = base_image
        .crop_imm(
            3 * base_image.width() / 4,
            0,
            base_image.width() / 4,
            base_image.height(),
        )
        .to_rgba32f();

    let sh_1s: [Vec<u8>; 3] = [
        (&*sh_0)
            .iter()
            .zip(&*sh_1_x)
            .map(|(&base, &float)| normalize_float(base, float))
            .collect(),
        (&*sh_0)
            .iter()
            .zip(&*sh_1_y)
            .map(|(&base, &float)| normalize_float(base, float))
            .collect(),
        (&*sh_0)
            .iter()
            .zip(&*sh_1_z)
            .map(|(&base, &float)| normalize_float(base, float))
            .collect(),
    ];

    {
        let output = args.next().unwrap();

        let half_bytes: Vec<u8> = (&*sh_0)
            .iter()
            .flat_map(|&float| f16::from_f32(float).to_le_bytes())
            .collect();

        let writer = ktx2_tools::Writer {
            header: ktx2_tools::WriterHeader {
                format: Some(ktx2::Format::R16G16B16A16_SFLOAT),
                type_size: 2,
                pixel_width: base_image.width() / 4,
                pixel_height: base_image.height(),
                pixel_depth: 1,
                layer_count: 0,
                face_count: 1,
                supercompression_scheme: Some(ktx2::SupercompressionScheme::Zstandard),
            },
            dfd_bytes: &4_u32.to_le_bytes(),
            key_value_pairs: &Default::default(),
            sgd_bytes: &[],
            uncompressed_levels_descending: &[Cow::Owned(half_bytes)],
        };
        writer
            .write(&mut std::fs::File::create(&output).unwrap())
            .unwrap();
    }

    for (output, bytes) in args.zip(sh_1s.into_iter()) {
        let writer = ktx2_tools::Writer {
            header: ktx2_tools::WriterHeader {
                format: Some(ktx2::Format::R8G8B8A8_UNORM),
                type_size: 1,
                pixel_width: base_image.width() / 4,
                pixel_height: base_image.height(),
                pixel_depth: 1,
                layer_count: 0,
                face_count: 1,
                supercompression_scheme: Some(ktx2::SupercompressionScheme::Zstandard),
            },
            dfd_bytes: &4_u32.to_le_bytes(),
            key_value_pairs: &Default::default(),
            sgd_bytes: &[],
            uncompressed_levels_descending: &[Cow::Owned(bytes)],
        };
        writer
            .write(&mut std::fs::File::create(&output).unwrap())
            .unwrap();
    }
}
