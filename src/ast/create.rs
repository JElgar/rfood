use syn::{Item, ItemEnum};

pub fn create_enum(name: &String, variants: Vec<syn::Variant>) -> syn::Item {
    Item::Enum(
        ItemEnum {
            attrs: [].to_vec(),
            vis: syn::Visibility::Inherited,
            enum_token: syn::token::Enum{
                span: syn::__private::Span::call_site(),
            },
            ident: syn::Ident::new(name, syn::__private::Span::call_site()),
            generics: syn::Generics {
                lt_token: None,
                params: syn::punctuated::Punctuated::from_iter(Vec::new() as Vec<syn::GenericParam>),
                gt_token: None,
                where_clause: None,
            },
            brace_token: syn::token::Brace{
                span: syn::__private::Span::call_site(),
            },
            variants: syn::punctuated::Punctuated::from_iter(variants),
        },
    )
}

pub fn create_enum_variant(name: &String, fields: syn::Fields) -> syn::Variant {
    syn::Variant{
        attrs: Vec::new() as Vec<syn::Attribute>,
        ident: syn::Ident::new(name, syn::__private::Span::call_site()),
        fields,
        discriminant: None,
    }
}

pub fn create_enum_unnamed_fields(fields: Vec<syn::Field>) -> syn::Fields {
    syn::Fields::Unnamed(
        syn::FieldsUnnamed {
            paren_token: syn::token::Paren{
                span: syn::__private::Span::call_site(),
            },
            unnamed: syn::punctuated::Punctuated::from_iter(fields),
        },
    )
}

// pub fn struct_to_enum_type(mut fields: &syn::Fields) -> &syn::Fields {
//     match fields {
//         syn::Fields::Named(internal_fields) => {
//             // If field is Box
//             let named_fields = internal_fields.named.iter().map(|field| {
//                 match &*field {
//                     syn::Field {
//                         ty: syn::Type::Path(
//                             syn::TypePath{
//                                 path: syn::Path{
//                                     segments,
//                                     ..
//                                 },
//                                 ..
//                             }
//                         ),
//                         ..
//                     } if match segments.first() {
//                         Some(syn::PathSegment{
//                             ident,
//                             ..
//                         }) => ident.to_string() == "Box",
//                         _ => false,
//                     } => field,
//                     _ => field,
//                 }
//             });
// 
//             internal_fields.named = syn::punctuated::Punctuated::from_iter(named_fields);
//             return fields;
//         },
//         syn::Fields::Unnamed(..) => panic!("Unnamed struct transform not supported"),
//         _ => panic!("Unsupported field format"),
//     }
// }

// pub fn create_enum_field() -> syn::Field {
//     syn::Field {
//         attrs: [].to_vec(),
//         vis: syn::Visibility::Inherited,
//         ident: None,
//         colon_token: None,
//         ty: syn::Type::Path(
//             syn::TypePath {
//                 qself: None,
//                 path: syn::Path {
//                     leading_colon: None,
//                     segments: [
//                         PathSegment {
//                             ident: Ident(
//                                 Box,
//                             ),
//                             arguments: AngleBracketed(
//                                 AngleBracketedGenericArguments {
//                                     colon2_token: None,
//                                     lt_token: Lt,
//                                     args: [
//                                         Type(
//                                             Path(
//                                                 TypePath {
//                                                     qself: None,
//                                                     path: Path {
//                                                         leading_colon: None,
//                                                         segments: [
//                                                             PathSegment {
//                                                                 ident: Ident(
//                                                                     Exp,
//                                                                 ),
//                                                                 arguments: None,
//                                                             },
//                                                         ],
//                                                     },
//                                                 },
//                                             ),
//                                         ),
//                                     ],
//                                     gt_token: Gt,
//                                 },
//                             ),
//                         },
//                     ],
//                 },
//             },
//         ),
//     }
// }
