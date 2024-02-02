use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::types::{self, MerkleTree};

pub struct Intermediate {
    pub types: Vec<TypeRef>,
    pub extrinsic_metadata: ExtrinsicMetadata,
}

/// A reference to a type in the registry.
pub type TypeRef = Rc<RefCell<TypeRefInner>>;

#[derive(Clone, Debug)]
pub enum TypeRefInner {
    Unresolved,
    Resolved(Type),
}

impl TypeRefInner {
    pub fn expect_resolved(&self) -> &Type {
        match self {
            Self::Resolved(t) => t,
            Self::Unresolved => panic!("Expected the `TypeRef` to be resolved"),
        }
    }

    pub fn resolved(&mut self, ty: Type) {
        *self = Self::Resolved(ty);
    }
}

#[derive(Clone, Debug)]
pub enum TypeDef {
    Composite(Vec<Field>),
    Enumeration(Vec<Variant>),
    Sequence(TypeRef),
    Array(TypeDefArray),
    Tuple(Vec<TypeRef>),
    Primitive(scale_info::TypeDefPrimitive),
    Compact(TypeRef),
    BitSequence(TypeDefBitSequence),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: Option<String>,
    pub ty: TypeRef,
    pub type_name: Option<String>,
}

impl Field {
    pub fn as_basic_type(&self) -> types::Field {
        types::Field {
            name: self.name.clone(),
            ty: self.ty.borrow().expect_resolved().as_basic_type_ref(),
            type_name: self.type_name.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
    pub index: u8,
}

impl Variant {
    pub fn as_basic_type(&self) -> types::TypeDefVariant {
        types::TypeDefVariant {
            name: self.name.clone(),
            fields: self.fields.iter().map(|f| f.as_basic_type()).collect(),
            index: self.index,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeDefArray {
    pub len: u32,
    pub type_param: TypeRef,
}

impl TypeDefArray {
    pub fn as_basic_type(&self) -> types::TypeDefArray {
        types::TypeDefArray {
            len: self.len,
            type_param: self
                .type_param
                .borrow()
                .expect_resolved()
                .as_basic_type_ref(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeDefBitSequence {
    pub bit_store_type: TypeRef,
    pub bit_order_type: TypeRef,
}

impl TypeDefBitSequence {
    pub fn as_basic_type(&self) -> types::TypeDefBitSequence {
        types::TypeDefBitSequence {
            bit_store_type: self
                .bit_store_type
                .borrow()
                .expect_resolved()
                .as_basic_type_ref(),
            bit_order_type: self
                .bit_order_type
                .borrow()
                .expect_resolved()
                .as_basic_type_ref(),
        }
    }
}

#[derive(Default)]
struct CollectPrimitives {
    found: Vec<scale_info::TypeDefPrimitive>,
}

impl Visitor for CollectPrimitives {
    fn visit_primitive(&mut self, primitive: &scale_info::TypeDefPrimitive) {
        self.found.push(primitive.clone());
    }
}

#[derive(Clone, Debug)]
pub struct Type {
    /// The unique path to the type. Can be empty for built-in types
    pub path: Vec<String>,
    /// The actual type definition
    pub type_def: TypeDef,
    pub unique_id: u32,
}

impl Type {
    pub fn as_basic_type(&self) -> Option<types::Type> {
        let mut collector = CollectPrimitives::default();
        collector.visit_type(&mut Default::default(), self);

        let type_def = match &self.type_def {
            TypeDef::Composite(_) | TypeDef::Tuple(_) if collector.found.is_empty() => return None,
            TypeDef::Compact(_) | TypeDef::Primitive(_) => return None,
            TypeDef::Enumeration(v) => {
                let mut variants = v.clone();
                variants.sort_by_key(|v| v.index);
                let variant_root_hash =
                    MerkleTree::calculate_root(variants.iter().map(|v| v.as_basic_type().hash()));
                types::TypeDef::Enumeration(variant_root_hash)
            }
            TypeDef::Array(a) => types::TypeDef::Array(a.as_basic_type()),
            TypeDef::Composite(c) => {
                types::TypeDef::Composite(c.iter().map(|f| f.as_basic_type()).collect())
            }
            TypeDef::Sequence(s) => {
                types::TypeDef::Sequence(s.borrow().expect_resolved().as_basic_type_ref())
            }
            TypeDef::Tuple(t) => types::TypeDef::Tuple(
                t.iter()
                    .map(|t| t.borrow().expect_resolved().as_basic_type_ref())
                    .collect(),
            ),
            TypeDef::BitSequence(b) => types::TypeDef::BitSequence(b.as_basic_type()),
        };

        Some(types::Type {
            path: self.path.clone(),
            type_def,
        })
    }

    pub fn as_basic_type_ref(&self) -> types::TypeRef {
        let mut collector = CollectPrimitives::default();
        collector.visit_type(&mut Default::default(), self);

        match &self.type_def {
            TypeDef::Primitive(p) => types::TypeRef::Primitive(match p {
                scale_info::TypeDefPrimitive::Bool => types::Primitives::Bool,
                scale_info::TypeDefPrimitive::Char => types::Primitives::Char,
                scale_info::TypeDefPrimitive::Str => types::Primitives::Str,
                scale_info::TypeDefPrimitive::U8 => types::Primitives::U8,
                scale_info::TypeDefPrimitive::U16 => types::Primitives::U16,
                scale_info::TypeDefPrimitive::U32 => types::Primitives::U32,
                scale_info::TypeDefPrimitive::U64 => types::Primitives::U64,
                scale_info::TypeDefPrimitive::U128 => types::Primitives::U128,
                scale_info::TypeDefPrimitive::U256 => types::Primitives::U256,
                scale_info::TypeDefPrimitive::I8 => types::Primitives::I8,
                scale_info::TypeDefPrimitive::I16 => types::Primitives::I16,
                scale_info::TypeDefPrimitive::I32 => types::Primitives::I32,
                scale_info::TypeDefPrimitive::I64 => types::Primitives::I64,
                scale_info::TypeDefPrimitive::I128 => types::Primitives::I128,
                scale_info::TypeDefPrimitive::I256 => types::Primitives::I256,
            }),
            TypeDef::Compact(_) => {
                let res = if collector.found.len() > 1 {
                    panic!("Unexpected: {:?}", collector.found)
                } else if let Some(found) = collector.found.first() {
                    match found {
                        scale_info::TypeDefPrimitive::U8 => types::Primitives::CompactU8,
                        scale_info::TypeDefPrimitive::U16 => types::Primitives::CompactU16,
                        scale_info::TypeDefPrimitive::U32 => types::Primitives::CompactU32,
                        scale_info::TypeDefPrimitive::U64 => types::Primitives::CompactU64,
                        scale_info::TypeDefPrimitive::U128 => types::Primitives::CompactU128,
                        p => panic!("Unsupported primitive type for `Compact`: {p:?}"),
                    }
                } else {
                    types::Primitives::Void
                };

                types::TypeRef::Primitive(res)
            }
            TypeDef::Tuple(_) | TypeDef::Composite(_) if collector.found.is_empty() => {
                types::TypeRef::Primitive(types::Primitives::Void)
            }
            _ => types::TypeRef::Ref(self.unique_id.into()),
        }
    }
}

pub trait Visitor {
    fn visit_type_def(&mut self, already_visited: &mut HashSet<u32>, type_def: &TypeDef) {
        visit_type_def(self, already_visited, type_def)
    }

    fn visit_type(&mut self, already_visited: &mut HashSet<u32>, ty: &Type) {
        visit_type(self, already_visited, ty)
    }

    fn visit_primitive(&mut self, _primitive: &scale_info::TypeDefPrimitive) {}
}

pub fn visit_type<V: Visitor + ?Sized>(
    visitor: &mut V,
    already_visited: &mut HashSet<u32>,
    ty: &Type,
) {
    visitor.visit_type_def(already_visited, &ty.type_def);
}

pub fn visit_type_def<V: Visitor + ?Sized>(
    visitor: &mut V,
    already_visited: &mut HashSet<u32>,
    type_def: &TypeDef,
) {
    match type_def {
        TypeDef::Enumeration(v) => {
            v.iter().for_each(|v| {
                for f in &v.fields {
                    if already_visited.insert(f.ty.borrow().expect_resolved().unique_id) {
                        visitor.visit_type(already_visited, f.ty.borrow().expect_resolved())
                    }
                }
            });
        }
        TypeDef::Array(a) => {
            if already_visited.insert(a.type_param.borrow().expect_resolved().unique_id) {
                visitor.visit_type(already_visited, a.type_param.borrow().expect_resolved())
            }
        }
        TypeDef::Composite(c) => {
            c.iter().for_each(|f| {
                if already_visited.insert(f.ty.borrow().expect_resolved().unique_id) {
                    visitor.visit_type(already_visited, f.ty.borrow().expect_resolved())
                }
            });
        }
        TypeDef::Sequence(s) => {
            if already_visited.insert(s.borrow().expect_resolved().unique_id) {
                visitor.visit_type(already_visited, s.borrow().expect_resolved())
            }
        }
        TypeDef::Tuple(t) => t.iter().for_each(|t| {
            if already_visited.insert(t.borrow().expect_resolved().unique_id) {
                visitor.visit_type(already_visited, t.borrow().expect_resolved())
            }
        }),
        TypeDef::Compact(c) => {
            if already_visited.insert(c.borrow().expect_resolved().unique_id) {
                visitor.visit_type(already_visited, c.borrow().expect_resolved())
            }
        }
        TypeDef::Primitive(p) => visitor.visit_primitive(p),
        TypeDef::BitSequence(b) => {
            if already_visited.insert(b.bit_order_type.borrow().expect_resolved().unique_id) {
                visitor.visit_type(already_visited, b.bit_order_type.borrow().expect_resolved())
            }

            if already_visited.insert(b.bit_store_type.borrow().expect_resolved().unique_id) {
                visitor.visit_type(already_visited, b.bit_store_type.borrow().expect_resolved())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExtrinsicMetadata {
    /// Extrinsic version.
    pub version: u8,
    pub address_ty: TypeRef,
    pub call_ty: TypeRef,
    pub signature_ty: TypeRef,
    /// The type of the extra data added to the extrinsic.
    pub extra_ty: TypeRef,
    /// The signed extensions in the order they appear in the extrinsic.
    pub signed_extensions: Vec<SignedExtensionMetadata>,
}

impl ExtrinsicMetadata {
    pub fn as_basic_type(&self) -> types::ExtrinsicMetadata {
        types::ExtrinsicMetadata {
            version: self.version,
            address_ty: self
                .address_ty
                .borrow()
                .expect_resolved()
                .as_basic_type_ref(),
            call_ty: self.call_ty.borrow().expect_resolved().as_basic_type_ref(),
            signature_ty: self
                .signature_ty
                .borrow()
                .expect_resolved()
                .as_basic_type_ref(),
            signed_extensions: self
                .signed_extensions
                .iter()
                .map(|se| se.as_basic_type())
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignedExtensionMetadata {
    pub identifier: String,
    pub included_in_extrinsic: TypeRef,
    pub included_in_signed_data: TypeRef,
}

impl SignedExtensionMetadata {
    pub fn as_basic_type(&self) -> types::SignedExtensionMetadata {
        types::SignedExtensionMetadata {
            identifier: self.identifier.clone(),
            ty: self
                .included_in_extrinsic
                .borrow()
                .expect_resolved()
                .as_basic_type_ref(),
            additional_signed: self
                .included_in_signed_data
                .borrow()
                .expect_resolved()
                .as_basic_type_ref(),
        }
    }
}
