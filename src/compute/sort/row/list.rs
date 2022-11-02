use crate::{
    array::{
        Array, BinaryArray, BooleanArray, FixedSizeListArray, ListArray, PrimitiveArray, Utf8Array,
    },
    compute::merge_sort::SortOptions,
    datatypes::PhysicalType,
    error::*,
    with_match_primitive_without_interval_type,
};

use super::{
    fixed,
    fixed::FixedLengthEncoding,
    null_sentinel,
    variable::{self, EMPTY_SENTINEL, NON_EMPTY_SENTINEL},
    Rows,
};

/// Compute the length of one list item.
pub fn encode_len(array: Option<Box<dyn Array>>) -> Result<usize> {
    match array {
        None => Ok(1),
        Some(list) => {
            let mut len = 1;
            match list.data_type().to_physical_type() {
                PhysicalType::Primitive(primitive) => {
                    with_match_primitive_without_interval_type!(primitive, |$T| {
                        let array = list
                            .as_any()
                            .downcast_ref::<PrimitiveArray<$T>>()
                            .unwrap();
                        len += fixed::encoded_len(array) * array.len();
                    })
                }
                PhysicalType::Null => {}
                PhysicalType::Boolean => len += bool::ENCODED_LEN * list.len(),
                PhysicalType::Binary => list
                    .as_any()
                    .downcast_ref::<BinaryArray<i32>>()
                    .unwrap()
                    .iter()
                    .for_each(|slice| len += variable::encoded_len(slice)),
                PhysicalType::LargeBinary => list
                    .as_any()
                    .downcast_ref::<BinaryArray<i64>>()
                    .unwrap()
                    .iter()
                    .for_each(|slice| len += variable::encoded_len(slice)),
                PhysicalType::Utf8 => list
                    .as_any()
                    .downcast_ref::<Utf8Array<i32>>()
                    .unwrap()
                    .iter()
                    .for_each(|slice| len += variable::encoded_len(slice.map(|x| x.as_bytes()))),
                PhysicalType::LargeUtf8 => list
                    .as_any()
                    .downcast_ref::<Utf8Array<i64>>()
                    .unwrap()
                    .iter()
                    .for_each(|slice| len += variable::encoded_len(slice.map(|x| x.as_bytes()))),
                PhysicalType::FixedSizeList => {
                    for a in list
                        .as_any()
                        .downcast_ref::<FixedSizeListArray>()
                        .unwrap()
                        .iter()
                    {
                        len += encode_len(a)?;
                    }
                }
                PhysicalType::List => {
                    for a in list
                        .as_any()
                        .downcast_ref::<ListArray<i32>>()
                        .unwrap()
                        .iter()
                    {
                        len += encode_len(a)?;
                    }
                }
                PhysicalType::LargeList => {
                    for a in list
                        .as_any()
                        .downcast_ref::<ListArray<i64>>()
                        .unwrap()
                        .iter()
                    {
                        len += encode_len(a)?;
                    }
                }
                t => {
                    return Err(Error::NotYetImplemented(format!(
                        "not yet implemented: {:?}",
                        t
                    )))
                }
            }
            Ok(len)
        }
    }
}

/// List types are encoded as
///
/// - single '0_u8' if null
/// - single '1_u8' if empty
/// - '2_u8' if not empty, followed by a series of encoded values
pub fn encode<I: Iterator<Item = Option<Box<dyn Array>>>>(out: &mut Rows, i: I, opts: SortOptions) {
    for (offset, maybe_val) in out.offsets.iter_mut().skip(1).zip(i) {
        match maybe_val {
            Some(list) if list.is_empty() => {
                out.buffer[*offset] = match opts.descending {
                    true => !EMPTY_SENTINEL,
                    false => EMPTY_SENTINEL,
                };
                *offset += 1;
            }
            Some(list) => {
                let end_offset = *offset + encode_len(Some(list.clone())).unwrap();
                let to_write = &mut out.buffer[*offset..end_offset];

                // Write `2_u8` to demarcate as non-empty, non-null array
                to_write[0] = NON_EMPTY_SENTINEL;

                match list.data_type().to_physical_type() {
                    PhysicalType::Primitive(primitive) => {
                        with_match_primitive_without_interval_type!(primitive, |$T| {
                            let column = list
                                .as_any()
                                .downcast_ref::<PrimitiveArray<$T>>()
                                .unwrap()
                                .iter()
                                .map(|v| v.map(|v| *v));
                            fixed::encode_raw(&mut to_write[1..], column);
                        })
                    }
                    PhysicalType::Null => {}
                    PhysicalType::Boolean => fixed::encode_raw(
                        &mut to_write[1..],
                        list.as_any().downcast_ref::<BooleanArray>().unwrap(),
                    ),
                    PhysicalType::Binary => {
                        variable::encode_raw(
                            &mut to_write[1..],
                            list.as_any()
                                .downcast_ref::<BinaryArray<i32>>()
                                .unwrap()
                                .iter(),
                        );
                    }
                    PhysicalType::LargeBinary => {
                        variable::encode_raw(
                            &mut to_write[1..],
                            list.as_any()
                                .downcast_ref::<BinaryArray<i64>>()
                                .unwrap()
                                .iter(),
                        );
                    }
                    PhysicalType::Utf8 => variable::encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<Utf8Array<i32>>()
                            .unwrap()
                            .iter()
                            .map(|x| x.map(|x| x.as_bytes())),
                    ),
                    PhysicalType::LargeUtf8 => variable::encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<Utf8Array<i64>>()
                            .unwrap()
                            .iter()
                            .map(|x| x.map(|x| x.as_bytes())),
                    ),
                    PhysicalType::FixedSizeList => encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<FixedSizeListArray>()
                            .unwrap()
                            .iter(),
                    ),
                    PhysicalType::List => encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<ListArray<i32>>()
                            .unwrap()
                            .iter(),
                    ),
                    PhysicalType::LargeList => encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<ListArray<i64>>()
                            .unwrap()
                            .iter(),
                    ),
                    t => unimplemented!("not yet implemented: {:?}", t),
                }

                *offset = end_offset;
                if opts.descending {
                    // Invert bits
                    to_write.iter_mut().for_each(|v| *v = !*v)
                }
            }
            None => {
                out.buffer[*offset] = null_sentinel(opts);
                *offset += 1;
            }
        }
    }
}

/// Encode without [`Rows`] and [`SortOptions`], just write to the buffer.
pub fn encode_raw<I: Iterator<Item = Option<Box<dyn Array>>>>(buffer: &mut [u8], i: I) {
    let mut offset = 0_usize;
    for maybe_val in i {
        match maybe_val {
            Some(list) if list.is_empty() => {
                buffer[offset] = EMPTY_SENTINEL;
                offset += 1;
            }
            Some(list) => {
                let end_offset = offset + encode_len(Some(list.clone())).unwrap();
                let to_write = &mut buffer[offset..end_offset];

                // Write `2_u8` to demarcate as non-empty, non-null array
                to_write[0] = NON_EMPTY_SENTINEL;

                match list.data_type().to_physical_type() {
                    PhysicalType::Primitive(primitive) => {
                        with_match_primitive_without_interval_type!(primitive, |$T| {
                            let column = list
                                .as_any()
                                .downcast_ref::<PrimitiveArray<$T>>()
                                .unwrap()
                                .iter()
                                .map(|v| v.map(|v| *v));
                            fixed::encode_raw(&mut to_write[1..], column);
                        })
                    }
                    PhysicalType::Null => {}
                    PhysicalType::Boolean => fixed::encode_raw(
                        &mut to_write[1..],
                        list.as_any().downcast_ref::<BooleanArray>().unwrap(),
                    ),
                    PhysicalType::Binary => {
                        variable::encode_raw(
                            &mut to_write[1..],
                            list.as_any()
                                .downcast_ref::<BinaryArray<i32>>()
                                .unwrap()
                                .iter(),
                        );
                    }
                    PhysicalType::LargeBinary => {
                        variable::encode_raw(
                            &mut to_write[1..],
                            list.as_any()
                                .downcast_ref::<BinaryArray<i64>>()
                                .unwrap()
                                .iter(),
                        );
                    }
                    PhysicalType::Utf8 => variable::encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<Utf8Array<i32>>()
                            .unwrap()
                            .iter()
                            .map(|x| x.map(|x| x.as_bytes())),
                    ),
                    PhysicalType::LargeUtf8 => variable::encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<Utf8Array<i64>>()
                            .unwrap()
                            .iter()
                            .map(|x| x.map(|x| x.as_bytes())),
                    ),
                    PhysicalType::FixedSizeList => encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<FixedSizeListArray>()
                            .unwrap()
                            .iter(),
                    ),
                    PhysicalType::List => encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<ListArray<i32>>()
                            .unwrap()
                            .iter(),
                    ),
                    PhysicalType::LargeList => encode_raw(
                        &mut to_write[1..],
                        list.as_any()
                            .downcast_ref::<ListArray<i64>>()
                            .unwrap()
                            .iter(),
                    ),
                    t => unimplemented!("not yet implemented: {:?}", t),
                }

                offset = end_offset;
            }
            None => {
                buffer[offset] = 0;
                offset += 1;
            }
        }
    }
}
