use crate::models::Vertex;

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.086_824_1, 0.492_403_86, 0.0], tex_coords: [0.413_175_9, 0.007_596_14] },
    Vertex { position: [-0.495_134_06, 0.069_586_47, 0.0], tex_coords: [0.004_865_944_4, 0.430_413_54] },
    Vertex { position: [-0.219_185_49, -0.449_397_06, 0.0], tex_coords: [0.280_814_53, 0.949_397] },
    Vertex { position: [0.359_669_98, -0.347_329_1, 0.0], tex_coords: [0.85967, 0.847_329_14] },
    Vertex { position: [0.441_473_72, 0.234_735_9, 0.0], tex_coords: [0.941_473_7, 0.265_264_1] },
];

pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];
