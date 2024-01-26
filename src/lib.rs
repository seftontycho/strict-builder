use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput};

fn extract_wrapped_type<'a>(ty: &'a syn::Type, wrapper: &str) -> Option<&'a syn::Type> {
    let path_segments = match ty {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path { segments, .. },
        }) => segments,
        _ => return None,
    };

    let segment = match path_segments.first() {
        Some(segment) => segment,
        None => return None,
    };

    if segment.ident != wrapper {
        return None;
    }

    let args = match segment.arguments {
        syn::PathArguments::AngleBracketed(ref args) => args,
        _ => return None,
    };

    if args.args.len() != 1 {
        return None;
    }

    match args.args.first() {
        Some(syn::GenericArgument::Type(ty)) => Some(ty),
        _ => return None,
    }
}

#[derive(Debug, Clone)]
enum Field<'a> {
    Required {
        ident: &'a syn::Ident,
        ty: &'a syn::Type,
    },
    Optional {
        ident: &'a syn::Ident,
        ty: &'a syn::Type,
    },
    Repeatable {
        ident: &'a syn::Ident,
        ty: &'a syn::Type,
        each_ident: syn::Ident,
    },
}

impl Field<'_> {
    fn parse_required(field: &syn::Field) -> Result<Option<Field>, syn::Error> {
        let ident = match &field.ident {
            Some(ident) => ident,
            None => return Err(syn::Error::new_spanned(field, "Expected named field")),
        };

        Ok(Some(Field::Required {
            ident: ident,
            ty: &field.ty,
        }))
    }

    fn parse_optional(field: &syn::Field) -> Result<Option<Field>, syn::Error> {
        let ident = match &field.ident {
            Some(ident) => ident,
            None => return Err(syn::Error::new_spanned(field, "Expected named field")),
        };

        if let Some(contained_type) = extract_wrapped_type(&field.ty, "Option") {
            return Ok(Some(Field::Optional {
                ident: &ident,
                ty: contained_type,
            }));
        };

        Ok(None)
    }

    fn parse_repeated(field: &syn::Field) -> Result<Option<Field>, syn::Error> {
        let ident = match &field.ident {
            Some(ident) => ident,
            None => return Err(syn::Error::new_spanned(field, "Expected named field")),
        };

        let each_ident = match parse_each(&field.attrs) {
            Ok(Some(ident)) => ident,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };

        match extract_wrapped_type(&field.ty, "Vec") {
            Some(contained_type) => Ok(Some(Field::Repeatable {
                ident: &ident,
                ty: contained_type,
                each_ident: each_ident,
            })),
            None => Err(syn::Error::new_spanned(
                field,
                format!("Expected Vec<T> for repeated field {}", stringify!(ident)),
            )),
        }
    }
}

fn parse_each(attrs: &Vec<syn::Attribute>) -> Result<Option<syn::Ident>, syn::Error> {
    let attr = match attrs.first() {
        Some(attr) => attr,
        None => return Ok(None),
    };

    if !attr.meta.path().is_ident("builder") {
        return Err(syn::Error::new_spanned(
            attr,
            "Expected #[builder(...)] attribute",
        ));
    };

    let tokens = match attr.meta {
        syn::Meta::List(syn::MetaList { ref tokens, .. }) => tokens,
        _ => {
            return Err(syn::Error::new_spanned(
                &attr.meta,
                "Expected #[builder(...)] attribute",
            ))
        }
    };

    let mut tokens = tokens.clone().into_iter();

    match tokens.next() {
        Some(proc_macro2::TokenTree::Ident(ident)) => {
            if ident != "each" {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "Expected #[builder(each = \"...\")]",
                ));
            }
        }
        _ => {
            return Err(syn::Error::new_spanned(
                &attr.meta,
                "Expected #[builder(each = \"...\")]",
            ))
        }
    };

    match tokens.next() {
        Some(proc_macro2::TokenTree::Punct(punct)) => {
            if punct.as_char() != '=' {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "Expected #[builder(each = \"...\")]",
                ));
            }
        }
        _ => {
            return Err(syn::Error::new_spanned(
                &attr.meta,
                "Expected #[builder(each = \"...\")]",
            ))
        }
    };

    let literal = match tokens.next() {
        Some(proc_macro2::TokenTree::Literal(literal)) => literal,
        _ => {
            return Err(syn::Error::new_spanned(
                &attr.meta,
                "Expected #[builder(each = \"...\")]",
            ))
        }
    };

    let each = literal.to_string();
    let each = format_ident!("{}", each.trim_matches('"'));

    Ok(Some(each))
}

fn parse_field(field: &syn::Field) -> Result<Field, syn::Error> {
    if let Some(field) = Field::parse_repeated(field)? {
        return Ok(field);
    }

    if let Some(field) = Field::parse_optional(field)? {
        return Ok(field);
    }

    if let Some(field) = Field::parse_required(field)? {
        return Ok(field);
    }

    Err(syn::Error::new_spanned(field, "Expected valid field"))
}

fn extract_fields(parsed: &DeriveInput) -> Result<&syn::Fields, syn::Error> {
    match &parsed.data {
        Data::Struct(DataStruct { fields, .. }) => Ok(fields),
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                parsed,
                "Expected Struct found Enum",
            ))
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                parsed,
                "Expected Struct found Union",
            ))
        }
    }
}

fn parse_fields(parsed: &DeriveInput) -> Result<Vec<Field>, syn::Error> {
    let fields = extract_fields(parsed)?;
    let fields = fields.iter().map(parse_field);

    if let Some(error) = combine_errors(fields.clone()) {
        return Err(error);
    }

    Ok(fields.into_iter().filter_map(|f| f.ok()).collect())
}

fn combine_errors<T>(errors: impl Iterator<Item = Result<T, syn::Error>>) -> Option<syn::Error> {
    let mut errors = errors.filter_map(|e| match e {
        Ok(_) => None,
        Err(e) => Some(e),
    });

    let first = errors.next()?;

    Some(errors.fold(first, |mut acc, e| {
        acc.combine(e);
        acc
    }))
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_strict_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as syn::DeriveInput);
    let output = _derive_strict_builder(parsed);

    match output {
        Ok(output) => output.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn _derive_strict_builder(
    parsed: syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let name = &parsed.ident;
    let builder_name = format_ident!("{}Builder", name);
    let fields = parse_fields(&parsed)?;

    let field_declarations = fields.iter().map(|field| match field {
        Field::Required { ident, ty } => {
            quote!(#ident: std::option::Option<#ty>)
        }
        Field::Optional { ident, ty } => {
            quote!(#ident: std::option::Option<#ty>)
        }
        Field::Repeatable { ident, ty, .. } => {
            quote!(#ident: std::vec::Vec<#ty>)
        }
    });

    let builder_struct = quote!(
        struct #builder_name {
            #(#field_declarations),*
        }
    );

    let field_defaults = fields.iter().map(|field| match field {
        Field::Required { ident, .. } => quote!(#ident: std::option::Option::None),
        Field::Optional { ident, .. } => quote!(#ident: std::option::Option::None),
        Field::Repeatable { ident, .. } => quote!(#ident: std::vec::Vec::new()),
    });

    let builder_func = quote!(
        impl #name {
            fn builder() -> #builder_name {
                #builder_name {
                    #(#field_defaults),*
                }
            }
        }
    );

    let populated_fields = fields.iter().map(|field| match field {
        Field::Required { ident, .. } => quote!(
            #ident: self.#ident.clone().ok_or(format!("missing required field: {}", stringify!(#ident)))?
        ),
        Field::Optional { ident, .. } => quote!(#ident: self.#ident.clone()),
        Field::Repeatable { ident, .. } => quote!(#ident: self.#ident.clone()),
    });

    let builder_functions = fields.iter().map(|field| match field {
        Field::Required { ident, ty } => {
            quote!(
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = std::option::Option::Some(#ident);
                    self
                }
            )
        }
        Field::Optional { ident, ty } => {
            quote!(
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = std::option::Option::Some(#ident);
                    self
                }
            )
        }
        Field::Repeatable {
            ident,
            ty,
            each_ident,
        } => {
            let each_fn = quote!(
                pub fn #each_ident(&mut self, #each_ident: #ty) -> &mut Self {
                    self.#ident.push(#each_ident);
                    self
                }
            );

            let set_fn = quote!(
                pub fn #ident(&mut self, #ident: std::vec::Vec<#ty>) -> &mut Self {
                    self.#ident = #ident;
                    self
                }
            );

            if ident == &each_ident {
                return each_fn;
            }

            quote!(
                #each_fn

                #set_fn
            )
        }
    });

    let builder_impl = quote!(
        impl #builder_name {
            pub fn build(&mut self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#populated_fields),*
                })
            }

            #(#builder_functions)*
        }
    );

    Ok(quote!(
        #builder_func

        #builder_struct

        #builder_impl
    ))
}
