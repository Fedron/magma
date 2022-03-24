#![recursion_limit = "128"]

use proc_macro::TokenStream;

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(PushConstantData)]
pub fn push_constant_data_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();
    let ident = &ast.ident;
    quote! {
        impl PushConstantData for #ident {
            fn as_bytes(&self) -> &[u8]
            where
                Self: Sized,
            {
                unsafe {
                    let size_in_bytes = std::mem::size_of::<Self>();
                    let size_in_u8 = size_in_bytes / std::mem::size_of::<u8>();
                    std::slice::from_raw_parts(self as *const Self as *const u8, size_in_u8)
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(Vertex)]
pub fn vertex_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = generate_vertex_impl(&ast);
    gen.into()
}

fn generate_vertex_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    quote! {
        impl Vertex for #ident {
            fn get_attribute_descriptions() -> Vec<VertexAttributeDescription> {
                vec![
                    VertexAttributeDescription {
                        binding: 0,
                        location: 0,
                        format: Format::R32G32B32_SFLOAT,
                        offset: offset_of!(Self, position) as u32,
                    },
                    VertexAttributeDescription {
                        binding: 0,
                        location: 1,
                        format: Format::R32G32B32_SFLOAT,
                        offset: offset_of!(Self, color) as u32,
                    },
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

// #[proc_macro_derive(Vertex, attributes(location))]
// pub fn vertex_derive(input: TokenStream) -> TokenStream {
//     let ast = syn::parse(input).unwrap();
//     let gen = generate_impl(&ast);
//     gen.into()
// }

// fn generate_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
//     let ident = &ast.ident;
//     let attribute_descriptions = generate_attribute_descriptions(&ast.data);

//     quote! {
//         impl Vertex for #ident {
//             fn get_attribute_descriptions() -> Vec<VertexAttributeDescription> {
//                 vec![#(#attribute_descriptions),*]
//             }

//             fn get_binding_descriptions() -> Vec<VertexBindingDescription> {
//                 vec![VertexBindingDescription {
//                     binding: 0,
//                     stride: std::mem::size_of::<SimpleVertex>() as u32,
//                     input_rate: VertexInputRate::VERTEX,
//                 }]
//             }
//         }
//     }
// }

// fn generate_attribute_descriptions(body: &syn::Data) -> Vec<proc_macro2::TokenStream> {
//     match body {
//         syn::Data::Enum(_) => panic!("Cannot implement Vertex on an enum"),
//         syn::Data::Union(_) => panic!("Cannot implement Vertex on a union"),
//         syn::Data::Struct(ref data) => data
//             .fields
//             .iter()
//             .map(generate_attribute_description)
//             .collect(),
//     }
// }

// fn generate_attribute_description(field: &syn::Field) -> proc_macro2::TokenStream {
//     let field_name = field.ident.as_ref().unwrap();
//     let format = vk_format_to_tokens(match &field.ty {
//         syn::Type::Array(array) => {
//             let is_valid_type = match array.elem.as_ref() {
//                 syn::Type::Path(path) => {
//                     let array_type = &path.path.segments.first().unwrap().ident;
//                     if array_type != "f32" {
//                         panic!(
//                             "Field {} has invalid type on array, only f32 is supported",
//                             field_name,
//                         )
//                     }

//                     true
//                 }
//                 _ => panic!("Field {} has invalid array element", field_name),
//             };
//             let array_len = match &array.len {
//                 syn::Expr::Lit(lit) => match &lit.lit {
//                     syn::Lit::Int(i) => i.base10_parse::<u32>().unwrap(),
//                     _ => panic!("Unexpected literal in array on field {}", field_name),
//                 },
//                 _ => panic!("Field {} has invalid expression in array", field_name),
//             };

//             match array_len {
//                 1 => ash::vk::Format::R32_SFLOAT,
//                 2 => ash::vk::Format::R32G32_SFLOAT,
//                 3 => ash::vk::Format::R32G32B32_SFLOAT,
//                 4 => ash::vk::Format::R32G32B32A32_SFLOAT,
//                 _ => panic!("Field {} has invalid array length", field_name),
//             }
//         }
//         _ => panic!("The type on field {} is not supported", field_name),
//     });

//     let location_attr = field
//         .attrs
//         .iter()
//         .filter(|a| a.path.is_ident("location"))
//         .next()
//         .unwrap_or_else(|| {
//             panic!(
//                 "Field {:?} is missing #[location = ?] attribute",
//                 field_name
//             )
//         });
//     let location_lit = match location_attr.parse_meta().unwrap() {
//         syn::Meta::NameValue(nv) => match nv.lit {
//             syn::Lit::Int(i) => i.base10_parse::<u32>().unwrap(),
//             _ => panic!(
//                 "Field {} location attribute value must be an integer",
//                 field_name
//             ),
//         },
//         _ => panic!(
//             "Field {} location attribute must be in the form #[location = ?]",
//             field_name
//         ),
//     };

//     quote! {
//         VertexAttributeDescription {
//             binding: 0,
//             location: #location_lit,
//             format: #format,
//             offset: offset_of!(Self, #field_name) as u32
//         }
//     }
// }

// fn vk_format_to_tokens(format: ash::vk::Format) -> proc_macro2::TokenStream {
//     match format {
//         ash::vk::Format::R32_SFLOAT => quote! { Format::R32_SFLOAT },
//         ash::vk::Format::R32G32_SFLOAT => quote! { Format::R32G32_SFLOAT },
//         ash::vk::Format::R32G32B32_SFLOAT => quote! { Format::R32G32B32_SFLOAT },
//         ash::vk::Format::R32G32B32A32_SFLOAT => quote! { Format::R32G32B32A32_SFLOAT },
//         _ => panic!("Invalid format"),
//     }
// }
