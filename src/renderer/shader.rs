use std::any::TypeId;
use std::fs::read_to_string;

use ash::vk;
use glsl::parser::Parse;
use glsl::syntax::{ExternalDeclaration, ShaderStage};
use magma_derive::UniformBuffer;

use crate::mesh::Vertex;

use super::UniformBufferDescription;

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

pub trait UniformBuffer {
    fn stage() -> vk::ShaderStageFlags;
    fn sizes() -> Vec<u32>;
    fn as_bytes(&self) -> &[u8];
}

#[derive(UniformBuffer)]
#[ubo(stage = "vertex")]
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

    pub fn check_vertex_attributes<V>(&self)
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
    }

    pub fn check_ubos(&self, ubos: Vec<UniformBufferDescription>) {
        // Represents the set, binding, and field sizes of every ubo in the shader
        let mut shader_ubos: Vec<(u32, u32, Vec<u32>)> = Vec::new();
        for declaration in self.declarations.iter() {
            if let glsl::syntax::ExternalDeclaration::Declaration(declaration) = declaration {
                if let glsl::syntax::Declaration::Block(block) = declaration {
                    let qualifiers = &block.qualifier.qualifiers.0;
                    for qualifier in qualifiers.iter() {
                        let mut ubo: Option<(u32, u32)> = None;
                        match qualifier {
                            glsl::syntax::TypeQualifierSpec::Layout(layout) => {
                                let mut set: Option<u32> = None;
                                let mut binding: Option<u32> = None;
                                for layout in layout.ids.0.iter() {
                                    if let glsl::syntax::LayoutQualifierSpec::Identifier(
                                        name,
                                        expr,
                                    ) = layout
                                    {
                                        if expr.is_none() {
                                            continue;
                                        }

                                        let expr = expr.as_ref().unwrap();
                                        if name.0 == "set" {
                                            if let glsl::syntax::Expr::IntConst(val) = **expr {
                                                set = Some(val as u32);
                                            }
                                        } else if name.0 == "binding" {
                                            if let glsl::syntax::Expr::IntConst(val) = **expr {
                                                binding = Some(val as u32);
                                            }
                                        }
                                    }
                                }

                                // Assume that a layout tha specifies a set and binding is a ubo
                                if set.is_some() && binding.is_some() {
                                    ubo = Some((set.unwrap(), binding.unwrap()));
                                }
                            }
                            _ => {}
                        }

                        if let Some(ubo) = ubo {
                            let mut sizes: Vec<u32> = Vec::new();
                            for field in block.fields.iter() {
                                let vk_type = glsl_type_to_vk(&field.ty.ty);
                                sizes.push(vk_type.1);
                            }
                            shader_ubos.push((ubo.0, ubo.1, sizes));
                        }
                    }
                }
            }
        }

        if shader_ubos.len() > 0 && ubos.len() == 0 {
            panic!(
                "Expected {} ubo(s) for shader, but found none in renderer",
                shader_ubos.len()
            );
        }

        if shader_ubos.len() != ubos.len() {
            panic!(
                "Shader and renderer define a different number of ubos, {} and {} respectively",
                shader_ubos.len(),
                ubos.len()
            );
        }

        for s_ubo in shader_ubos.iter() {
            let mut found_ubo = false;
            for ubo in ubos.iter() {
                if !(s_ubo.0 == ubo.set && s_ubo.1 == ubo.binding) {
                    continue;
                }

                found_ubo = true;

                for (index, (&s_size, u_size)) in s_ubo.2.iter().zip(ubo.sizes.clone()).enumerate()
                {
                    if s_size != u_size {
                        panic!("Field {} doesn't match size in bytes in the shader with the ubo supplied in the renderer (expected = {}, found = {})", index, s_size, u_size);
                    }
                }
            }

            if !found_ubo {
                panic!("Shader expects a ubo at (set = {}, binding = {}) but couldn't find a ubo with the same set and binding in the renderer", s_ubo.0, s_ubo.1);
            }
        }
    }

    pub fn check_push_constant<P: 'static>(&self)
    where
        P: UniformBuffer,
    {
        for declaration in self.declarations.iter() {
            if let glsl::syntax::ExternalDeclaration::Declaration(declaration) = declaration {
                if let glsl::syntax::Declaration::Block(block) = declaration {
                    let qualifiers = &block.qualifier.qualifiers.0;
                    let mut is_push_constant = false;
                    for qualifier in qualifiers.iter() {
                        match qualifier {
                            glsl::syntax::TypeQualifierSpec::Layout(layout) => {
                                for layout in layout.ids.0.iter() {
                                    if let glsl::syntax::LayoutQualifierSpec::Identifier(name, _) =
                                        layout
                                    {
                                        if name.0 == "push_constant" {
                                            is_push_constant = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }

                        if is_push_constant {
                            break;
                        }
                    }

                    if is_push_constant {
                        if TypeId::of::<P>() == TypeId::of::<NonePushConstant>() {
                            panic!("Found push constant in shader, but trying to build renderer without push constant");
                        }

                        let mut push_sizes: Vec<u32> = Vec::new();
                        for field in block.fields.iter() {
                            let vk_type = glsl_type_to_vk(&field.ty.ty);
                            push_sizes.push(vk_type.1);
                        }

                        let push_size: u32 = push_sizes.iter().sum();
                        if push_size > 128 {
                            panic!(
                            "Push constant can't be larger that 128 bytes, found size of {} bytes",
                            push_size
                        );
                        }

                        let mut sizes: Vec<u32> = Vec::new();
                        for field in block.fields.iter() {
                            sizes.push(glsl_type_to_vk(&field.ty.ty).1);
                        }

                        if P::sizes().iter().sum::<u32>() > 128 {
                            panic!("PushConstantData struct is larger than 128 bytes which exceeds the max size");
                        }

                        if sizes.len() != P::sizes().len() {
                            panic!("Shader push constant has more fields that provided PushConstantData struct");
                        }

                        for (index, (&size, p_size)) in sizes.iter().zip(P::sizes()).enumerate() {
                            if size != p_size {
                                panic!(
                                    "Field {} has a different size between the PushConstantData ({}) struct and shader ({})",
                                    index,
                                    p_size,
                                    size
                                );
                            }
                        }

                        break;
                    }
                }
            }
        }
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
        glsl::syntax::TypeSpecifierNonArray::Vec4 => (vk::Format::R32G32B32A32_SFLOAT, 16),
        glsl::syntax::TypeSpecifierNonArray::Mat4 => (vk::Format::UNDEFINED, 64),
        _ => panic!("Unsupported type '{:?}'", ty),
    }
}
