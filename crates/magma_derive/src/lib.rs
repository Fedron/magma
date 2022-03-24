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
                    stride: std::mem::size_of::<Self>() as u32,
                    input_rate: VertexInputRate::VERTEX,
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
            format: Format::R32G32B32_SFLOAT,
            offset: offset_of!(Self, #field_name) as u32
        }
    }
}
