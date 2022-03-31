use std::fs::read_to_string;

use ash::vk;
use glsl::parser::Parse;
use glsl::syntax::{ExternalDeclaration, ShaderStage};
use magma_derive::PushConstant;

use crate::mesh::Vertex;

#[derive(Debug, Clone, Copy)]
pub struct Shader {
    pub file: &'static str,
    pub entry_point: &'static str,
    pub stage: vk::ShaderStageFlags,
}

impl Shader {
    pub const VERTEX: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
    pub const FRAGMENT: vk::ShaderStageFlags = vk::ShaderStageFlags::FRAGMENT;
}

pub trait PushConstant {
    fn stage() -> vk::ShaderStageFlags;
    fn as_bytes(&self) -> &[u8];
}

#[derive(PushConstant)]
#[push_constant(stage = "vertex")]
pub struct NonePushConstant;

#[derive(Debug, Clone, Copy)]
struct VertexAttribute {
    pub ty: vk::Format,
    pub size: u32,
    pub location: u32,
    pub offset: u32,
}

impl Into<vk::VertexInputAttributeDescription> for VertexAttribute {
    fn into(self) -> vk::VertexInputAttributeDescription {
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: self.location,
            format: self.ty,
            offset: self.offset,
        }
    }
}

pub struct ShaderCompiler {
    declarations: Vec<ExternalDeclaration>,
}

impl ShaderCompiler {
    pub fn new(shader: Shader) -> ShaderCompiler {
        ShaderCompiler {
            declarations: ShaderStage::parse(
                read_to_string(shader.file).expect("Failed to read shader file"),
            )
            .expect("Failed to parse shader")
            .0
             .0,
        }
    }

    pub fn check_vertex_attributes<V>(self) -> ShaderCompiler
    where
        V: Vertex,
    {
        let mut vertex_attributes: Vec<VertexAttribute> = Vec::new();
        for declaration in self.declarations.iter() {
            if let glsl::syntax::ExternalDeclaration::Declaration(declaration) = declaration {
                if let glsl::syntax::Declaration::InitDeclaratorList(attribute) = declaration {
                    let attribute = &attribute.head.ty;

                    let qualifiers = attribute.qualifier.as_ref().unwrap();
                    if let glsl::syntax::TypeQualifierSpec::Storage(storage) =
                        qualifiers.qualifiers.0.get(1).unwrap()
                    {
                        if *storage != glsl::syntax::StorageQualifier::In {
                            continue;
                        }

                        if let glsl::syntax::TypeQualifierSpec::Layout(layout) =
                            qualifiers.qualifiers.0.first().unwrap()
                        {
                            // TODO: Check that the identifier is a "location"
                            if let glsl::syntax::LayoutQualifierSpec::Identifier(_, expr) =
                                layout.ids.0.first().unwrap()
                            {
                                let expr = expr.as_ref().unwrap();
                                if let glsl::syntax::Expr::IntConst(val) = **expr {
                                    let vulkan_ty = glsl_type_to_vk(&attribute.ty.ty);
                                    vertex_attributes.push(VertexAttribute {
                                        ty: vulkan_ty.0,
                                        size: vulkan_ty.1,
                                        location: val as u32,
                                        offset: calculate_offset(&vertex_attributes),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        let vertex_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> =
            vertex_attributes
                .clone()
                .into_iter()
                .map(|attribute| attribute.into())
                .collect();

        let vertex_binding_descriptions = vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: calculate_stride(&vertex_attributes),
            input_rate: vk::VertexInputRate::VERTEX,
        }];

        if vertex_attribute_descriptions.len() != V::get_attribute_descriptions().len() {
            panic!("Shader has a different amount of attributes compared to the Vertex");
        }

        for attribute in vertex_attribute_descriptions.iter() {
            let mut found: Option<vk::VertexInputAttributeDescription> = None;
            for v_attribute in V::get_attribute_descriptions() {
                if v_attribute.location == attribute.location {
                    found = Some(v_attribute);
                    break;
                }
            }

            if let Some(v_attribute) = found {
                if attribute.binding == v_attribute.binding
                    && attribute.location == v_attribute.location
                    && attribute.format == v_attribute.format
                    && attribute.offset == v_attribute.offset
                {
                    continue;
                }

                panic!(
                "Shader has the following attribute, but the same could not be found in the Vertex struct: {:#?}",
                attribute
            );
            }
        }

        if V::get_binding_descriptions().len() != 1 {
            panic!(
                "Expected for Vertex struct to have one binding, instead found {}",
                V::get_binding_descriptions().len()
            );
        }

        let binding = vertex_binding_descriptions.first().unwrap();
        let v_binding = V::get_binding_descriptions();
        let v_binding = v_binding.first().unwrap();

        if binding.binding != v_binding.binding
            && binding.stride != v_binding.stride
            && binding.input_rate != v_binding.input_rate
        {
            panic!(
                "Shader has the following binding '{:#?}' but Vertex has a binding of '{:#?}'",
                binding, v_binding
            );
        }

        self
    }
}

fn calculate_offset(attributes: &[VertexAttribute]) -> u32 {
    let mut offset = 0;
    for attribute in attributes.iter() {
        offset += attribute.size;
    }
    offset
}

fn calculate_stride(attributes: &[VertexAttribute]) -> u32 {
    let mut stride = 0;
    for attribute in attributes.iter() {
        stride += attribute.size;
    }
    stride
}

fn glsl_type_to_vk(ty: &glsl::syntax::TypeSpecifierNonArray) -> (vk::Format, u32) {
    match ty {
        glsl::syntax::TypeSpecifierNonArray::Vec2 => (vk::Format::R32G32_SFLOAT, 8),
        glsl::syntax::TypeSpecifierNonArray::Vec3 => (vk::Format::R32G32B32_SFLOAT, 12),
        glsl::syntax::TypeSpecifierNonArray::Mat4 => (vk::Format::UNDEFINED, 64),
        _ => panic!("Unsupported type '{:?}'", ty),
    }
}
