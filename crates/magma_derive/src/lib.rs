//! This crate provides useful derive macros for various `magma` functionality

#![recursion_limit = "128"]

use darling::{usage::Options, FromDeriveInput};
use proc_macro::TokenStream;

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(push_constant))]
struct PushConstantOpts {
    stage: Option<String>,
}

/// Implements the [`as_bytes`] function for [`PushConstant`]. Required if you want
/// to be able to pass your [`PushConstant`] to a [`RenderPipeline`].
#[proc_macro_derive(PushConstant, attributes(push_constant))]
pub fn push_constant_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();
    let ident = &ast.ident;

    let options = PushConstantOpts::from_derive_input(&ast)
        .expect("Provided invalid options, only expected 'stage'");
    let stage = options
        .stage
        .expect("Didn't provide a stage")
        .to_lowercase();
    let stage = stage.as_str();
    let stage = match stage {
        "vertex" => quote! { Shader::VERTEX },
        "fragment" => quote! { Shader::FRAGMENT },
        _ => panic!("Unsupported shader stage, must be one of ['vertex', 'fragment']"),
    };

    quote! {
        impl PushConstant for #ident {
            fn as_bytes(&self) -> &[u8] {
                unsafe {
                    let size_in_bytes = std::mem::size_of::<Self>();
                    let size_in_u8 = size_in_bytes / std::mem::size_of::<u8>();
                    std::slice::from_raw_parts(self as *const Self as *const u8, size_in_u8)
                }
            }

            fn stage() -> ash::vk::ShaderStageFlags {
                #stage
            }
        }
    }
    .into()
}

/// Generates implementation for getting a [`Vertex`]'s attribute and binding descriptions.
/// Only works on structs, and expects all attributes to be arrays of type f32 or i32.
///
/// Each attribute should be labeled with a `#[location = ?]` which should match the layout location
/// in the shader the [`Vertex`] will be used in. This is not checked and is up to you to ensure.
#[proc_macro_derive(Vertex, attributes(location))]
pub fn vertex_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = generate_vertex_impl(&ast);
    gen.into()
}

/// Generates the main `impl` body
fn generate_vertex_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    let attribute_descriptions = generate_attribute_descriptions(&ast.data);

    quote! {
        impl Vertex for #ident {
            fn get_attribute_descriptions() -> Vec<VertexAttributeDescription> {
                vec![
                    #(#attribute_descriptions),*
                ]
            }

            fn get_binding_descriptions() -> Vec<VertexBindingDescription> {
                vec![VertexBindingDescription {
                    binding: 0,
                    stride: std::mem::size_of::<Self>() as u32,
                    input_rate: VertexInputRate::VERTEX,
                }]
            }
        }
    }
}

/// Generates the attribute descriptions for each attribute in the struct
fn generate_attribute_descriptions(body: &syn::Data) -> Vec<proc_macro2::TokenStream> {
    match body {
        syn::Data::Enum(_) => panic!("Cannot implement Vertex on an enum"),
        syn::Data::Union(_) => panic!("Cannot implement Vertex on a union"),
        syn::Data::Struct(ref data) => data
            .fields
            .iter()
            .map(generate_attribute_description)
            .collect(),
    }
}

/// Generates a [`VertexAttributeDescription`] for a field
fn generate_attribute_description(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_type = get_field_type(field);

    let location_attr = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("location"))
        .next()
        .unwrap_or_else(|| {
            panic!(
                "Field {:?} is missing #[location = ?] attribute",
                field_name
            )
        });
    let location_lit = match location_attr.parse_meta().unwrap() {
        syn::Meta::NameValue(nv) => match nv.lit {
            syn::Lit::Int(i) => i.base10_parse::<u32>().unwrap(),
            _ => panic!(
                "Field {} location attribute value must be an integer",
                field_name
            ),
        },
        _ => panic!(
            "Field {} location attribute must be in the form #[location = ?]",
            field_name
        ),
    };

    quote! {
        VertexAttributeDescription {
            binding: 0,
            location: #location_lit,
            format: #field_type,
            offset: offset_of!(Self, #field_name) as u32
        }
    }
}

/// Converts a rust type to the corresponding [`Format`]
///
/// Currently only array types are supported and are expected to be either f32 or i32
fn get_field_type(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident.as_ref().unwrap();
    match &field.ty {
        syn::Type::Array(array) => {
            let array_type = match &*array.elem {
                syn::Type::Path(path) => path
                    .path
                    .get_ident()
                    .expect("Failed to get ident of array")
                    .to_string(),
                _ => panic!("Failed too read array type on field {}", field_name),
            };

            let array_len = match &array.len {
                syn::Expr::Lit(lit) => match &lit.lit {
                    syn::Lit::Int(i) => i.base10_parse::<u32>().unwrap(),
                    _ => panic!("Field {} had unexpected literal in array", field_name),
                },
                _ => panic!("Field {} had unexpected literal in array", field_name),
            };

            if array_type.eq("f32".into()) {
                match array_len {
                    1 => quote! { Format::R32_SFLOAT },
                    2 => quote! { Format::R32G32_SFLOAT },
                    3 => quote! { Format::R32G32B32_SFLOAT },
                    4 => quote! { Format::R32G32B32A32_SFLOAT },
                    _ => panic!(
                        "Field {} has invalid array length, should be 1, 2, 3, or 4",
                        field_name
                    ),
                }
            } else if array_type.eq("i32".into()) {
                match array_len {
                    1 => quote! { Format::R32_SINT },
                    2 => quote! { Format::R32G32_SINT },
                    3 => quote! { Format::R32G32B32_SINT },
                    4 => quote! { Format::R32G32B32A32_SINT },
                    _ => panic!(
                        "Field {} has invalid array length, should be 1, 2, 3, or 4",
                        field_name
                    ),
                }
            } else {
                panic!(
                    "Field {} has an invalid array type, should be f32 or i32",
                    field_name
                );
            }
        }
        _ => panic!("Field {} should be an array, float or int type", field_name),
    }
}
