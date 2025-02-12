use crate::utils::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedVec3 {
    data: u128,
}

impl From<Vec3> for PackedVec3 {
    fn from(value: Vec3) -> Self {
        value.as_dvec3().into()
    }
}

impl From<DVec3> for PackedVec3 {
    fn from(value: DVec3) -> Self {
        const SCALE: f64 = 16384.0;

        let ints = value.floor();
        let decs = (value - ints) * SCALE;

        let x = ints.x as i64 * 16384 + decs.x as i64;
        let y = ints.y as i64 * 16384 + decs.y as i64;
        let z = ints.z as i64 * 16384 + decs.z as i64;

        let x_packed = u64::from_ne_bytes((x & ((1 << 43) - 1)).to_ne_bytes()) as u128;
        let y_packed = u64::from_ne_bytes((y & ((1 << 43) - 1)).to_ne_bytes()) as u128;
        let z_packed = u64::from_ne_bytes((z & ((1 << 42) - 1)).to_ne_bytes()) as u128;

        let data = (x_packed << (43 + 42)) | (y_packed << 42) | z_packed;
        PackedVec3 { data }
    }
}

impl From<PackedVec3> for UVec4 {
    fn from(value: PackedVec3) -> Self {
        let bytes = value.data.to_ne_bytes();
        uvec4(
            u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            u32::from_ne_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            u32::from_ne_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        )
    }
}
