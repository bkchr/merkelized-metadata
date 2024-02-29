use std::collections::{BTreeMap, BTreeSet};

use codec::{Compact, Decode, Input};
use scale_decode::{
    ext::scale_type_resolver::{
        BitsOrderFormat, BitsStoreFormat, Primitive as RPrimitive, ResolvedTypeVisitor, Variant,
    },
    visitor::{decode_with_visitor, DecodeError},
    Field, TypeResolver, Visitor,
};

use crate::{
    from_frame_metadata::TypeInformation,
    types::{TypeDef, TypeRef},
};

impl TypeResolver for TypeInformation {
    type TypeId = TypeRef;

    type Error = String;

    fn resolve_type<'this, V: ResolvedTypeVisitor<'this, TypeId = TypeRef>>(
        &'this self,
        type_id: &TypeRef,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let type_id = match type_id {
            TypeRef::ById(id) => id.0,
            TypeRef::Bool => return Ok(visitor.visit_primitive(RPrimitive::Bool)),
            TypeRef::Char => return Ok(visitor.visit_primitive(RPrimitive::Char)),
            TypeRef::Str => return Ok(visitor.visit_primitive(RPrimitive::Str)),
            TypeRef::U8 => return Ok(visitor.visit_primitive(RPrimitive::U8)),
            TypeRef::U16 => return Ok(visitor.visit_primitive(RPrimitive::U16)),
            TypeRef::U32 => return Ok(visitor.visit_primitive(RPrimitive::U32)),
            TypeRef::U64 => return Ok(visitor.visit_primitive(RPrimitive::U64)),
            TypeRef::U128 => return Ok(visitor.visit_primitive(RPrimitive::U128)),
            TypeRef::U256 => return Ok(visitor.visit_primitive(RPrimitive::U256)),
            TypeRef::I8 => return Ok(visitor.visit_primitive(RPrimitive::I8)),
            TypeRef::I16 => return Ok(visitor.visit_primitive(RPrimitive::I16)),
            TypeRef::I32 => return Ok(visitor.visit_primitive(RPrimitive::I32)),
            TypeRef::I64 => return Ok(visitor.visit_primitive(RPrimitive::I64)),
            TypeRef::I128 => return Ok(visitor.visit_primitive(RPrimitive::I128)),
            TypeRef::I256 => return Ok(visitor.visit_primitive(RPrimitive::I256)),
            TypeRef::CompactU8 => return Ok(visitor.visit_compact(&TypeRef::U8)),
            TypeRef::CompactU16 => return Ok(visitor.visit_compact(&TypeRef::U16)),
            TypeRef::CompactU32 => return Ok(visitor.visit_compact(&TypeRef::U32)),
            TypeRef::CompactU64 => return Ok(visitor.visit_compact(&TypeRef::U64)),
            TypeRef::CompactU128 => return Ok(visitor.visit_compact(&TypeRef::U128)),
            TypeRef::Void => return Ok(visitor.visit_composite(core::iter::empty())),
        };

        let types = self
            .types
            .get(&type_id)
            .ok_or_else(|| format!("Unknown type id {type_id}"))?;

        if types.is_empty() {
            return Err(format!("{type_id} type is empty"));
        }

        let type_def = types[0].type_def;
        let value = match &type_def {
            TypeDef::Array(a) => visitor.visit_array(&a.type_param, a.len as usize),
            TypeDef::Composite(c) => visitor.visit_composite(c.iter().map(|f| Field {
                name: f.name.as_deref(),
                id: &f.ty,
            })),
            TypeDef::Enumeration(_) => visitor.visit_variant(types.iter().map(|t| {
                let TypeDef::Enumeration(v) = t.type_def else {
                    panic!("AHH")
                };

                Variant {
                    index: v.index,
                    name: &v.name,
                    fields: v.fields.iter().map(|f| Field {
                        name: f.name.as_deref(),
                        id: &f.ty,
                    }),
                }
            })),
            TypeDef::Sequence(s) => visitor.visit_sequence(s),
            TypeDef::Tuple(t) => visitor.visit_tuple(t.iter()),
            TypeDef::BitSequence(b) => {
                let store_format = match b.num_bytes {
                    1 => BitsStoreFormat::U8,
                    2 => BitsStoreFormat::U16,
                    4 => BitsStoreFormat::U32,
                    8 => BitsStoreFormat::U64,
                    b => {
                        return Err(format!(
                            "Unsupported number of bytes {b} for type {type_id}"
                        ))
                    }
                };

                let bit_order = if b.least_significant_bit_first {
                    BitsOrderFormat::Lsb0
                } else {
                    BitsOrderFormat::Msb0
                };

                visitor.visit_bit_sequence(store_format, bit_order)
            }
        };

        Ok(value)
    }
}

#[derive(Clone)]
enum AccessedType {
    Enumeration(BTreeSet<u32>),
    Other,
}

impl AccessedType {
    fn add_variant(&mut self, variant: u32) {
        if let Self::Enumeration(variants) = self {
            variants.insert(variant);
        } else {
            panic!("`add_variant` should only be called for `Enumeration`s.")
        }
    }
}

#[derive(Clone, Default)]
struct CollectAccessedTypes {
    accessed_types: BTreeMap<u32, AccessedType>,
}

impl Visitor for CollectAccessedTypes {
    type TypeResolver = TypeInformation;
    type Value<'scale, 'resolver> = Self;
    type Error = DecodeError;

    fn visit_bool<'scale, 'resolver>(
        mut self,
        _value: bool,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_char<'scale, 'resolver>(
        mut self,
        _value: char,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_u8<'scale, 'resolver>(
        mut self,
        _value: u8,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_u16<'scale, 'resolver>(
        mut self,
        _value: u16,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_u32<'scale, 'resolver>(
        mut self,
        _value: u32,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_u64<'scale, 'resolver>(
        mut self,
        _value: u64,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_u128<'scale, 'resolver>(
        mut self,
        _value: u128,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_u256<'scale, 'resolver>(
        mut self,
        _value: &'scale [u8; 32],
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_i8<'scale, 'resolver>(
        mut self,
        _value: i8,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_i16<'scale, 'resolver>(
        mut self,
        _value: i16,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_i32<'scale, 'resolver>(
        mut self,
        _value: i32,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_i64<'scale, 'resolver>(
        mut self,
        _value: i64,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_i128<'scale, 'resolver>(
        mut self,
        _value: i128,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_i256<'scale, 'resolver>(
        mut self,
        _value: &'scale [u8; 32],
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_sequence<'scale, 'resolver>(
        mut self,
        value: &mut scale_decode::visitor::types::Sequence<'scale, 'resolver, Self::TypeResolver>,
        type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.accessed_types.insert(
            type_id
                .id()
                .expect("Sequence is always referenced by id; qed"),
            AccessedType::Other,
        );

        value.decode_item(self.clone()).unwrap_or(Ok(self))
    }

    fn visit_composite<'scale, 'resolver>(
        mut self,
        value: &mut scale_decode::visitor::types::Composite<'scale, 'resolver, Self::TypeResolver>,
        type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.accessed_types.insert(
            type_id
                .id()
                .expect("Composite is always referenced by id; qed"),
            AccessedType::Other,
        );

        let mut visitor = self;
        while let Some(v) = value.decode_item(visitor.clone()) {
            visitor = v?;
        }

        Ok(visitor)
    }

    fn visit_tuple<'scale, 'resolver>(
        mut self,
        value: &mut scale_decode::visitor::types::Tuple<'scale, 'resolver, Self::TypeResolver>,
        type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.accessed_types.insert(
            type_id.id().expect("Tuple is always referenced by id; qed"),
            AccessedType::Other,
        );

        let mut visitor = self;
        while let Some(v) = value.decode_item(visitor.clone()) {
            visitor = v?;
        }

        Ok(visitor)
    }

    fn visit_str<'scale, 'resolver>(
        mut self,
        _value: &mut scale_decode::visitor::types::Str<'scale>,
        _type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        Ok(self)
    }

    fn visit_variant<'scale, 'resolver>(
        mut self,
        value: &mut scale_decode::visitor::types::Variant<'scale, 'resolver, Self::TypeResolver>,
        type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.accessed_types
            .entry(
                type_id
                    .id()
                    .expect("Enumeration is always referenced by id; qed"),
            )
            .or_insert_with(|| AccessedType::Enumeration(Default::default()))
            .add_variant(value.index() as u32);

        let mut visitor = self;
        while let Some(v) = value.fields().decode_item(visitor.clone()) {
            visitor = v?;
        }

        Ok(visitor)
    }

    fn visit_array<'scale, 'resolver>(
        mut self,
        value: &mut scale_decode::visitor::types::Array<'scale, 'resolver, Self::TypeResolver>,
        type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.accessed_types.insert(
            type_id
                .id()
                .expect("BitSequence is always referenced by id; qed"),
            AccessedType::Other,
        );

        let mut visitor = self;
        while let Some(v) = value.decode_item(visitor.clone()) {
            visitor = v?;
        }
        Ok(visitor)
    }

    fn visit_bitsequence<'scale, 'resolver>(
        mut self,
        _value: &mut scale_decode::visitor::types::BitSequence<'scale>,
        type_id: &TypeRef,
    ) -> Result<Self::Value<'scale, 'resolver>, Self::Error> {
        self.accessed_types.insert(
            type_id
                .id()
                .expect("BitSequence is always referenced by id; qed"),
            AccessedType::Other,
        );
        Ok(self)
    }
}

pub fn decode_extrinsic_and_collect_type_ids(
    mut extrinsic: &[u8],
    mut additional_signed: Option<&[u8]>,
    type_information: &TypeInformation,
) -> Result<Vec<(u32, Vec<u32>)>, String> {
    let _length = Compact::<u32>::decode(&mut extrinsic)
        .map_err(|e| format!("Failed to read length: {e}"))?;

    let version = (&mut extrinsic)
        .read_byte()
        .map_err(|e| format!("Failed to read version byte: {e}"))?;

    let is_signed = version & 0b1000_0000 != 0;
    let version = version & 0b0111_1111;
    if version != 4 {
        return Err("Invalid transaction version".into());
    }

    let visitor = is_signed
        .then(|| {
            let visitor = decode_with_visitor(
                &mut extrinsic,
                &type_information.extrinsic_metadata.address_ty,
                type_information,
                CollectAccessedTypes::default(),
            )
            .map_err(|e| format!("Failed to decode address: {e}"))?;

            let visitor = decode_with_visitor(
                &mut extrinsic,
                &type_information.extrinsic_metadata.signature_ty,
                type_information,
                visitor,
            )
            .map_err(|e| format!("Failed to decode signature: {e}"))?;

            type_information
                .extrinsic_metadata
                .signed_extensions
                .iter()
                .try_fold(visitor, |visitor, se| {
                    decode_with_visitor(
                        &mut extrinsic,
                        &se.included_in_extrinsic,
                        type_information,
                        visitor,
                    )
                    .map_err(|e| format!("Failed to decode extra ({}): {e}", se.identifier))
                })
        })
        .transpose()?
        .unwrap_or_default();

    let visitor = decode_with_visitor(
        &mut extrinsic,
        &type_information.extrinsic_metadata.call_ty,
        type_information,
        visitor,
    )
    .map_err(|e| format!("Failed to decode signature: {e}"))?;

    let visitor = additional_signed
        .map(|mut additional| {
            type_information
                .extrinsic_metadata
                .signed_extensions
                .iter()
                .try_fold(visitor, |visitor, se| {
                    decode_with_visitor(
                        &mut additional,
                        &se.included_in_extrinsic,
                        type_information,
                        visitor,
                    )
                    .map_err(|e| format!("Failed to decode extra ({}): {e}", se.identifier))
                })
        })
        .unwrap_or_else(|| Ok(visitor))?;

    Ok(visitor
        .accessed_types
        .into_iter()
        .map(|(id, ty)| match ty {
            AccessedType::Other => (id, Vec::new()),
            AccessedType::Enumeration(variants) => (id, variants.into_iter().collect()),
        })
        .collect())
}
