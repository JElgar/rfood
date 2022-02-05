pub fn create_enum(name: &String) -> syn::Item {
    syn::Item::Enum(
        syn::ItemEnum {
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
            variants: syn::punctuated::Punctuated::from_iter(Vec::new() as Vec<syn::Variant>),
        },
    )
}

// pub fn create_varaint() -> syn::Item::Variant {
//                     variants: [
//                         Variant {
//                             attrs: [],
//                             ident: Ident(
//                                 Lit,
//                             ),
//                             fields: Unnamed(
//                                 FieldsUnnamed {
//                                     paren_token: Paren,
//                                     unnamed: [
//                                         Field {
//                                             attrs: [],
//                                             vis: Inherited,
//                                             ident: None,
//                                             colon_token: None,
//                                             ty: Path(
//                                                 TypePath {
//                                                     qself: None,
//                                                     path: Path {
//                                                         leading_colon: None,
//                                                         segments: [
//                                                             PathSegment {
//                                                                 ident: Ident(
//                                                                     i32,
//                                                                 ),
//                                                                 arguments: None,
//                                                             },
//                                                         ],
//                                                     },
//                                                 },
//                                             ),
//                                         },
//                                     ],
//                                 },
//                             ),
//                             discriminant: None,
//                         },
//                         Comma,
//                         Variant {
//                             attrs: [],
//                             ident: Ident(
//                                 Sub,
//                             ),
//                             fields: Unnamed(
//                                 FieldsUnnamed {
//                                     paren_token: Paren,
//                                     unnamed: [
//                                         Field {
//                                             attrs: [],
//                                             vis: Inherited,
//                                             ident: None,
//                                             colon_token: None,
//                                             ty: Path(
//                                                 TypePath {
//                                                     qself: None,
//                                                     path: Path {
//                                                         leading_colon: None,
//                                                         segments: [
//                                                             PathSegment {
//                                                                 ident: Ident(
//                                                                     Box,
//                                                                 ),
//                                                                 arguments: AngleBracketed(
//                                                                     AngleBracketedGenericArguments {
//                                                                         colon2_token: None,
//                                                                         lt_token: Lt,
//                                                                         args: [
//                                                                             Type(
//                                                                                 Path(
//                                                                                     TypePath {
//                                                                                         qself: None,
//                                                                                         path: Path {
//                                                                                             leading_colon: None,
//                                                                                             segments: [
//                                                                                                 PathSegment {
//                                                                                                     ident: Ident(
//                                                                                                         Exp,
//                                                                                                     ),
//                                                                                                     arguments: None,
//                                                                                                 },
//                                                                                             ],
//                                                                                         },
//                                                                                     },
//                                                                                 ),
//                                                                             ),
//                                                                         ],
//                                                                         gt_token: Gt,
//                                                                     },
//                                                                 ),
//                                                             },
//                                                         ],
//                                                     },
//                                                 },
//                                             ),
//                                         },
//                                         Comma,
//                                         Field {
//                                             attrs: [],
//                                             vis: Inherited,
//                                             ident: None,
//                                             colon_token: None,
//                                             ty: Path(
//                                                 TypePath {
//                                                     qself: None,
//                                                     path: Path {
//                                                         leading_colon: None,
//                                                         segments: [
//                                                             PathSegment {
//                                                                 ident: Ident(
//                                                                     Box,
//                                                                 ),
//                                                                 arguments: AngleBracketed(
//                                                                     AngleBracketedGenericArguments {
//                                                                         colon2_token: None,
//                                                                         lt_token: Lt,
//                                                                         args: [
//                                                                             Type(
//                                                                                 Path(
//                                                                                     TypePath {
//                                                                                         qself: None,
//                                                                                         path: Path {
//                                                                                             leading_colon: None,
//                                                                                             segments: [
//                                                                                                 PathSegment {
//                                                                                                     ident: Ident(
//                                                                                                         Exp,
//                                                                                                     ),
//                                                                                                     arguments: None,
//                                                                                                 },
//                                                                                             ],
//                                                                                         },
//                                                                                     },
//                                                                                 ),
//                                                                             ),
//                                                                         ],
//                                                                         gt_token: Gt,
//                                                                     },
//                                                                 ),
//                                                             },
//                                                         ],
//                                                     },
//                                                 },
//                                             ),
//                                         },
//                                     ],
//                                 },
//                             ),
//                             discriminant: None,
//                         },
//                         Comma,
//                     ],
// }
