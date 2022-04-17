#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use darling::FromDeriveInput;
use proc_macro::TokenStream;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(ubo))]
struct UniformBufferOpts {
    stage: Option<String>,
}

#[proc_macro_derive(UniformBuffer, attributes(ubo))]
pub fn derive_uniform_buffer(input: TokenStream) -> TokenStream {
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();
    let ident = &ast.ident;

    let opts = UniformBufferOpts::from_derive_input(&ast)
        .expect("Provided unvalid options, only expected `stage`");
    let stage = opts.stage.expect("No stage provided").to_lowercase();
    let stage = stage.as_str();
    let stage = match stage {
        "vertex" => quote! { ShaderStageFlags::VERTEX },
        "fragment" => quote! { ShaderStageFlags::FRAGMENT },
        "compute" => quote! { ShaderStageFlags::COMPUTE },
        "all_graphics" => quote! { ShaderStageFlags::ALL_GRAPHICS },
        _ => panic!("Unsupported shader stage, must be one of ['vertex', 'fragment', 'compute', 'all_graphics']")
    };

    let descriptions = generate_field_descriptions(&ast.data);

    quote! {
        impl UniformBuffer for #ident {
            /// TODO: Correct alignment automatically
            ///
            /// Scalars need to be aligned by N (= 4 bytes)
            /// Vec2 needs to be aligned by 2N (= 8 bytes)
            /// Vec3 and vec4 need to be aligned by 4N (= 16 bytes)
            /// A nested struct needs the base alignment of each member rounded to a multiple of 16
            /// A mat4 needs to have the same alignment as a vec4
            fn as_bytes(&self) -> &[u8] {
                unsafe {
                    let size_in_bytes = ::std::mem::size_of::<Self>();
                    let size_in_u8 = size_in_bytes / ::std::mem::size_of::<u8>();
                    std::slice::from_raw_parts(self as *const Self as *const u8, size_in_u8)
                }
            }

            fn stage() -> ShaderStageFlags {
                #stage
            }

            fn get_field_descriptions() -> Vec<UboFieldDescription> {
                #descriptions
            }
        }
    }
    .into()
}

fn generate_field_descriptions(data: &syn::Data) -> proc_macro2::TokenStream {
    let mut sizes: Vec<usize> = Vec::new();

    match data {
        syn::Data::Struct(data) => {
            for field in data.fields.iter() {
                sizes.push(get_field_size(field));
            }

            let mut descriptions: Vec<proc_macro2::TokenStream> = Vec::new();
            for &size in sizes.iter() {
                descriptions.push(quote! {
                    UboFieldDescription {
                        size: #size
                    }
                });
            }

            quote! {
                vec![
                    #(#descriptions),*
                ]
            }
        }
        _ => panic!("Only a struct derive UniformBuffer"),
    }
}

fn get_field_size(field: &syn::Field) -> usize {
    let field_name = &field.ident.as_ref().unwrap();
    match &field.ty {
        syn::Type::Path(ref path) => {
            if path.path.segments.len() == 2 {
                if path
                    .path
                    .segments
                    .first()
                    .unwrap()
                    .ident
                    .to_string()
                    .as_str()
                    == "glam"
                {
                    let ty = path.path.segments[1].ident.to_string();
                    let ty = ty.as_str();
                    match ty {
                        "Vec2" => 8,
                        "Vec3" => 12,
                        "Vec4" => 16,
                        "Mat2" => 16,
                        "Mat3" => 36,
                        "Mat4" => 64,
                        _ => panic!("Field `{}` has unsupported glam type `{}`", field_name, ty)
                    }
                } else {
                    panic!(
                        "Field `{}` has a type with 2 path segments but only types from glam are supported (e.g. glam::Vec3)",
                        field_name
                    );
                }
            } else if path.path.segments.len() == 1 {
                let ty = format!("{}", path.path.get_ident().unwrap());
                let ty = ty.as_str();
                match ty {
                    "u32" => ::std::mem::size_of::<u32>(),
                    _ => panic!("Type `{}` on field `{}` is not supported", ty, field_name),
                }
            } else {
                panic!(
                    "Field `{}` has a type with more than 2 path segments, only built-in types like `[f32; 3]` or glam types like `glam::Vec3` are supported",
                    field_name
                );
            }
        }
        syn::Type::Array(array) => {
            let array_type = match &*array.elem {
                syn::Type::Path(path) => path
                    .path
                    .get_ident()
                    .expect("Failed to get ident of array")
                    .to_string(),
                _ => panic!("Failed to read array type on field `{}`", field_name),
            };

            let array_len = match &array.len {
                syn::Expr::Lit(lit) => match &lit.lit {
                    syn::Lit::Int(i) => i.base10_parse::<u32>().unwrap(),
                    _ => panic!(
                        "Field `{}` has unexpected literal in array type",
                        field_name
                    ),
                },
                _ => panic!(
                    "Field `{}` has unexpected literal in array type",
                    field_name
                ),
            };

            if array_type.ne("f32") {
                panic!("Field `{}` must be an f32 array", field_name);
            }

            (4 * array_len) as usize
        }
        _ => panic!("Field `{}` has unsupported type", field_name),
    }
}

#[proc_macro_derive(Vertex, attributes(location))]
pub fn vertex_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = generate_vertex_impl(&ast);
    gen.into()
}

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
                    stride: ::std::mem::size_of::<Self>() as u32,
                    input_rate: VertexInputRate::Vertex,
                }]
            }
        }
    }
}

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
                "Field `{:?}` is missing #[location = ?] attribute",
                field_name
            )
        });
    let location_lit = match location_attr.parse_meta().unwrap() {
        syn::Meta::NameValue(nv) => match nv.lit {
            syn::Lit::Int(i) => i.base10_parse::<u32>().unwrap(),
            _ => panic!(
                "Field `{:?}` location attribute must be an integer",
                field_name
            ),
        },
        _ => panic!(
            "Field `{:?}` location attribute must be in the form #[location = ?]",
            field_name
        ),
    };

    quote! {
        VertexAttributeDescription {
            binding: 0,
            location: #location_lit,
            format: #field_type,
            offset: offset_of!(Self, #field_name) as u32,
        }
    }
}

fn get_field_type(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident.as_ref().unwrap();
    match &field.ty {
        syn::Type::Array(array) => {
            let array_type = match &*array.elem {
                syn::Type::Path(path) => path
                    .path
                    .get_ident()
                    .expect("Failed to get identifier of array")
                    .to_string(),
                _ => panic!("Failed to read array type of field `{}`", field_name),
            };
            let array_len = match &array.len {
                syn::Expr::Lit(lit) => match &lit.lit {
                    syn::Lit::Int(i) => i.base10_parse::<u32>().unwrap(),
                    _ => panic!("Field `{}` has unexpected literal in array", field_name),
                },
                _ => panic!("Field `{}` has unexpected literal in array", field_name),
            };

            if array_type.eq("f32".into()) {
                match array_len {
                    1 => quote! { VkFormat::R32_SFLOAT },
                    2 => quote! { VkFormat::R32G32_SFLOAT },
                    3 => quote! { VkFormat::R32G32B32_SFLOAT },
                    4 => quote! { VkFormat::R32G32B32A32_SFLOAT },
                    _ => panic!(
                        "Field `{}` has invalid array length, should be 1, 2, 3, or 4",
                        field_name
                    ),
                }
            } else if array_type.eq("i32".into()) {
                match array_len {
                    1 => quote! { VkFormat::R32_SINT },
                    2 => quote! { VkFormat::R32G32_SINT },
                    3 => quote! { VkFormat::R32G32B32_SINT },
                    4 => quote! { VkFormat::R32G32B32A32_SINT },
                    _ => panic!(
                        "Field `{}` has invalid array length, should be 1, 2, 3, or 4",
                        field_name
                    ),
                }
            } else {
                panic!(
                    "Field `{}` has an invalid array type, should be f32 or i32",
                    field_name
                );
            }
        }
        _ => panic!("Field `{}` should be an array type", field_name),
    }
}
