use polars::datatypes::AnyValue;

// dumb, but works
pub fn get_u8(v: AnyValue<'_>) -> u8 {
    match v {
        AnyValue::UInt8(v) => v,
        _ => panic!(),
    }
}

pub fn get_u16(v: AnyValue<'_>) -> u16 {
    match v {
        AnyValue::UInt16(v) => v,
        _ => panic!(),
    }
}

pub fn get_u32(v: AnyValue<'_>) -> u32 {
    match v {
        AnyValue::UInt32(v) => v,
        _ => panic!(),
    }
}

pub fn get_f32(v: AnyValue<'_>) -> f32 {
    match v {
        AnyValue::Float32(v) => v,
        _ => panic!(),
    }
}

pub fn get_str(v: AnyValue<'_>) -> &str {
    match v {
        AnyValue::String(v) => v,
        _ => panic!(),
    }
}
