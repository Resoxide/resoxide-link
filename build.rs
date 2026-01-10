use std::env;
use std::path::PathBuf;
use proc_macro2::{Span,TokenStream};
use quote::quote;

static PASCAL_CASE_OPTIONS: stringcase::Options = stringcase::Options {
    separate_before_non_alphabets: false,
    separate_after_non_alphabets: false,
    separators: "",
    keep: "",
};

fn pascal_case(input: &str) -> String {
    stringcase::pascal_case_with_options(input, &PASCAL_CASE_OPTIONS)
}

static DIM_NAMES: [&'static str;4] = ["x", "y", "z", "w"];

fn matrix(base_type: &str, ty: &str) -> TokenStream {
    let rust_ty = syn::Ident::new(ty,Span::call_site());
    let mut tokens = TokenStream::new();
    for dim in 2..=4 {
        let pascal_base = pascal_case(base_type);
        let matrix_name = format!("{pascal_base}{dim}x{dim}");
        let matrix_ty = syn::Ident::new(&matrix_name, Span::call_site());
        let mut fields = TokenStream::new();
        let mut identity = TokenStream::new();
        let zero = zero_for(ty);
        let one = one_for(ty);
        for c in 0..dim {
            for r in 0..dim {
                let dim_name = format!("m{c}{r}");
                let dim_ident = syn::Ident::new(&dim_name, Span::call_site());
                fields.extend(quote! {
                    pub #dim_ident: #rust_ty,
                });
                identity.extend(if c == r {
                    quote! { #dim_ident: #one, }
                } else {
                    quote! { #dim_ident: #zero, }
                });
            }
        }
        tokens.extend(quote! {
            #[derive(Clone,Copy,Default,Json,Debug,PartialEq)]
            #[repr(C)]
            pub struct #matrix_ty {
                #fields
            }

            impl #matrix_ty {
                pub const IDENTITY: #matrix_ty = #matrix_ty { #identity };
            }
        })
    }
    tokens
}

fn has_eq(ty: &str) -> bool {
    ["i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize","bool"].contains(&ty)
}

fn one_for(ty: &str) -> syn::Lit {
    if ["i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize"].contains(&ty) {
        let lit = format!("1{ty}");
        syn::Lit::Int(syn::LitInt::new(&lit, Span::call_site()))
    } else if ["f32","f64"].contains(&ty) {
        let lit = format!("1.0{ty}");
        syn::Lit::Float(syn::LitFloat::new(&lit, Span::call_site()))
    } else if ["bool"].contains(&ty) {
        syn::Lit::Bool(syn::LitBool::new(true, Span::call_site()))
    } else {
        let lit = format!("{ty}::from(1)");
        syn::Lit::Verbatim(proc_macro2::Literal::string(&lit))
    }
}

fn zero_for(ty: &str) -> syn::Lit {
    if ["i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize"].contains(&ty) {
        let lit = format!("0{ty}");
        syn::Lit::Int(syn::LitInt::new(&lit, Span::call_site()))
    } else if ["f32","f64"].contains(&ty) {
        let lit = format!("0.0{ty}");
        syn::Lit::Float(syn::LitFloat::new(&lit, Span::call_site()))
    } else if ["bool"].contains(&ty) {
        syn::Lit::Bool(syn::LitBool::new(false, Span::call_site()))
    } else {
        let lit = format!("{ty}::from(1)");
        syn::Lit::Verbatim(proc_macro2::Literal::string(&lit))
    }
}

fn glam_type(ty: &str, glam: &str) -> Option<String> {
    match ty {
        "f32" => Some(format!("{glam}")),
        "f64" => Some(format!("D{glam}")),
        "i8" => Some(format!("I8{glam}")),
        "i16" => Some(format!("I16{glam}")),
        "i32" => Some(format!("I{glam}")),
        "i64" => Some(format!("I64{glam}")),
        "u8" => Some(format!("U8{glam}")),
        "u16" => Some(format!("U16{glam}")),
        "u32" => Some(format!("U{glam}")),
        "u64" => Some(format!("U64{glam}")),
        "bool" => Some(format!("B{glam}")),
        _ => None,
    }
}

fn vector(base_type: &str, ty: &str) -> TokenStream {
    let rust_ty = syn::Ident::new(ty,Span::call_site());
    let mut tokens = TokenStream::new();
    for dim in 2..=4 {
        let pascal_base = pascal_case(base_type);
        let vector_name = format!("{pascal_base}{dim}");
        let vector_ty = syn::Ident::new(&vector_name, Span::call_site());
        let mut fields = TokenStream::new();
        let mut accessors = TokenStream::new();
        let mut assignment_from_v = TokenStream::new();
        let mut assignment_from_parameters = TokenStream::new();
        let mut parameters = TokenStream::new();
        let mut ones = TokenStream::new();
        let one = one_for(ty);
        let mut zeroes = TokenStream::new();
        let zero = zero_for(ty);
        for i in 0..dim {
            let dim_ident = syn::Ident::new(DIM_NAMES[i], Span::call_site());
            fields.extend(quote! {
                pub #dim_ident: #rust_ty,
            });
            accessors.extend(quote! {
                v.#dim_ident,
            });
            assignment_from_v.extend(quote! {
                #dim_ident: v.#dim_ident,
            });
            assignment_from_parameters.extend(quote! {
                #dim_ident,
            });
            parameters.extend(quote! {
                #dim_ident: #rust_ty,
            });
            zeroes.extend(quote! { #dim_ident: #zero, });
            ones.extend(quote! { #dim_ident: #one, });
        }
        let mut eq = TokenStream::new();
        if has_eq(ty) {
            eq.extend(quote! {,Eq,Hash});
        }
        tokens.extend(quote! {
            #[derive(Clone,Copy,Default,Json,Debug,PartialEq #eq)]
            #[repr(C)]
            pub struct #vector_ty {
                #fields
            }

            impl #vector_ty {
                pub const ZEROES: #vector_ty = #vector_ty { #zeroes };
                pub const ONES: #vector_ty = #vector_ty { #ones };

                pub fn new(#parameters) -> Self {
                    Self { #assignment_from_parameters }
                }
            }
        });
        if let Some(glam) = glam_type(ty, &*format!("Vec{dim}")) {
            let glam_ty = syn::Ident::new(&glam, Span::call_site());
            let glam_func = syn::Ident::new(&*glam.to_lowercase(), Span::call_site());
            tokens.extend(quote! {
                #[cfg(feature = "glam")]
                impl From<#vector_ty> for ::glam::#rust_ty::#glam_ty {
                    fn from(v: #vector_ty) -> Self {
                        ::glam::#rust_ty::#glam_func(#accessors)
                    }
                }

                #[cfg(feature = "glam")]
                impl From<::glam::#rust_ty::#glam_ty> for #vector_ty {
                    fn from(v: ::glam::#rust_ty::#glam_ty) -> Self {
                        Self { #assignment_from_v }
                    }
                }
            })
        }
    }
    tokens
}

fn quaternion(base_type: &str, ty: &str) -> TokenStream {
    let rust_ty = syn::Ident::new(ty,Span::call_site());
    let mut tokens = TokenStream::new();
    let pascal_base = pascal_case(base_type);
    let quaternion_name = format!("{pascal_base}Q");
    let quaternion_ty = syn::Ident::new(&quaternion_name, Span::call_site());
    let mut fields = TokenStream::new();
    for i in 0..4 {
        let dim_ident = syn::Ident::new(DIM_NAMES[i], Span::call_site());
        fields.extend(quote! {
            pub #dim_ident: #rust_ty,
        });
    }
    tokens.extend(quote! {
        #[derive(Clone,Copy,Debug,Json,PartialEq)]
        #[repr(C)]
        pub struct #quaternion_ty {
            #fields
        }

        impl Default for #quaternion_ty {
            fn default() -> Self {
                Self {
                    x: 0.0, y: 0.0, z: 0.0, w: 1.0,
                }
            }
        }
    });
    tokens
}

fn field(ty_name: &str, ty: &str) -> TokenStream {
    let pascal_name = pascal_case(ty_name);
    let rust_ty = syn::Ident::new(ty, Span::call_site());
    let variant = syn::Ident::new(&pascal_name, Span::call_site());
    let nullable_variant = syn::Ident::new(&*format!("Nullable{pascal_name}"), Span::call_site());
    let array_ty = syn::Ident::new(&*format!("Array{pascal_name}"), Span::call_site());
    let field_ty = syn::Ident::new(&*format!("Field{pascal_name}"), Span::call_site());
    let nullable_field_ty = syn::Ident::new(&*format!("FieldNullable{pascal_name}"), Span::call_site());

    quote! {
        #[derive(Debug,Default,Json)]
        pub struct #field_ty {
            pub id: Option<String>,
            pub value: #rust_ty,
        }
        #[derive(Debug,Default,Json)]
        pub struct #array_ty {
            pub id: Option<String>,
            pub values: Vec<#rust_ty>,
        }
        #[derive(Debug,Default,Json)]
        pub struct #nullable_field_ty {
            pub id: Option<String>,
            pub value: Option<#rust_ty>,
        }
        impl From<#rust_ty> for Member {
            fn from(value: #rust_ty) -> Self {
                Self::#variant(#field_ty { id: None, value })
            }
        }
        impl From<Option<#rust_ty>> for Member {
            fn from(value: Option<#rust_ty>) -> Self {
                Self::#nullable_variant(#nullable_field_ty { id: None, value: value })
            }
        }
        impl From<Vec<#rust_ty>> for Member {
            fn from(values: Vec<#rust_ty>) -> Self {
                Self::#array_ty(#array_ty { id: None, values })
            }
        }
    }
}

fn variant_nullable(ty_name: &str) -> TokenStream {
    let pascal_name = pascal_case(ty_name);
    let struct_ty = syn::Ident::new(&*format!("Field{pascal_name}"), Span::call_site());
    let variant = syn::Ident::new(&*pascal_name, Span::call_site());
    let nullable_struct_ty = syn::Ident::new(&*format!("FieldNullable{pascal_name}"), Span::call_site());
    let nullable_variant = syn::Ident::new(&*format!("Nullable{pascal_name}"), Span::call_site());
    let array_variant = syn::Ident::new(&*format!("Array{pascal_name}"), Span::call_site());
    let nullable_discriminator = format!("{ty_name}?");
    let array_discriminator = format!("{ty_name}[]");
    quote! {
        #[json(rename = #ty_name)]
        #variant(#struct_ty),
        #[json(rename = #nullable_discriminator)]
        #nullable_variant(#nullable_struct_ty),
        #[json(rename = #array_discriminator)]
        #array_variant(#array_variant),
    }
}

fn ref_variant_nullable(ty_name: &str) -> TokenStream {
    let pascal_name = pascal_case(ty_name);
    let struct_ty = syn::Ident::new(&*format!("Field{pascal_name}"), Span::call_site());
    let variant = syn::Ident::new(&*pascal_name, Span::call_site());
    let nullable_struct_ty = syn::Ident::new(&*format!("FieldNullable{pascal_name}"), Span::call_site());
    let nullable_variant = syn::Ident::new(&*format!("Nullable{pascal_name}"), Span::call_site());
    let array_variant = syn::Ident::new(&*format!("Array{pascal_name}"), Span::call_site());
    let nullable_discriminator = format!("{ty_name}?");
    let array_discriminator = format!("{ty_name}[]");
    quote! {
        #[json(rename = #ty_name)]
        #variant(&'a #struct_ty),
        #[json(rename = #nullable_discriminator)]
        #nullable_variant(&'a #nullable_struct_ty),
        #[json(rename = #array_discriminator)]
        #array_variant(&'a #array_variant),
    }
}

fn impl_from(ty_name: &str, nullable: bool) -> TokenStream {
    let pascal_name = pascal_case(ty_name);
    let field_ty = syn::Ident::new(&*format!("Field{pascal_name}"), Span::call_site());
    let array_ty = syn::Ident::new(&*format!("Array{pascal_name}"), Span::call_site());
    let variant = syn::Ident::new(&*pascal_name, Span::call_site());
    let nullable_field_ty = syn::Ident::new(&*format!("FieldNullable{pascal_name}"), Span::call_site());
    let nullable_variant = syn::Ident::new(&*format!("Nullable{pascal_name}"), Span::call_site());
    if nullable {
        quote! {
            impl From<#array_ty> for Member {
                fn from(value: #array_ty) -> Self {
                    Self::#array_ty(value)
                }
            }

            impl From<#nullable_field_ty> for Member {
                fn from(value: #nullable_field_ty) -> Self {
                    Self::#nullable_variant(value)
                }
            }

            impl From<#field_ty> for Member {
                fn from(value: #field_ty) -> Self {
                    Self::#variant(value)
                }
            }
        }
    } else {
        quote! {
            impl From<#field_ty> for Member {
                fn from(value: #field_ty) -> Self {
                    Self::#variant(value)
                }
            }

            impl From<#array_ty> for Member {
                fn from(value: #array_ty) -> Self {
                    Self::#array_ty(value)
                }
            }
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let vector_types = [
        "float",
        "double",
        "byte",
        "ushort",
        "uint",
        "ulong",
        "sbyte",
        "short",
        "int",
        "long",
        "bool",
    ];

    let complex_types = [
        "float",
        "double",
    ];

    let types = [
        ("byte","u8"),
        ("ushort","u16"),
        ("uint","u32"),
        ("ulong","u64"),
        ("sbyte","i8"),
        ("short","i16"),
        ("int","i32"),
        ("long","i64"),
        ("float","f32"),
        ("double","f64"),
        ("decimal","Decimal"),
        ("bool","bool"),
        ("char","Char"),
        ("color","Color"),
        ("colorX","ColorX"),
        ("color32","Color32"),
    ];

    let mut type_stream = TokenStream::new();
    let mut variant_stream = TokenStream::new();
    let mut ref_variant_stream = TokenStream::new();
    let mut impl_stream = TokenStream::new();
    for &(name, ty) in types.iter() {
        type_stream.extend(field(name, ty));
        variant_stream.extend(variant_nullable(name));
        ref_variant_stream.extend(ref_variant_nullable(name));
        impl_stream.extend(impl_from(name, true));
        if vector_types.contains(&name) {
            type_stream.extend(vector(name, ty));
            for dim in 2..=4 {
                let name_dim = format!("{name}{dim}");
                let ty_dim = pascal_case(&name_dim);
                type_stream.extend(field(&name_dim, &ty_dim));
                variant_stream.extend(variant_nullable(&name_dim));
                ref_variant_stream.extend(ref_variant_nullable(&name_dim));
                impl_stream.extend(impl_from(&name_dim, true));
            }
        }
        if complex_types.contains(&name) {
            type_stream.extend(quaternion(name, ty));
            let quaternion_name = format!("{name}Q");
            let ty_name = pascal_case(&quaternion_name);
            type_stream.extend(field(&quaternion_name,&ty_name));
            variant_stream.extend(variant_nullable(&quaternion_name));
            ref_variant_stream.extend(ref_variant_nullable(&quaternion_name));
            impl_stream.extend(impl_from(&quaternion_name, true));
            type_stream.extend(matrix(name, ty));
            for dim in 2..=4 {
                let name_dim = format!("{name}{dim}x{dim}");
                let ty_dim = pascal_case(&name_dim);
                type_stream.extend(field(&name_dim, &ty_dim));
                variant_stream.extend(variant_nullable(&name_dim));
                ref_variant_stream.extend(ref_variant_nullable(&name_dim));
                impl_stream.extend(impl_from(&name_dim, true));
            }
        }
    }

    type_stream.extend(quote! {
        #[derive(Debug,Default,Json)]
        pub struct FieldString {
            pub id: Option<String>,
            pub value: Option<String>,
        }
        #[derive(Debug,Default,Json)]
        pub struct ArrayString {
            pub id: Option<String>,
            pub values: Vec<Option<String>>,
        }
        #[derive(Debug,Default,Json)]
        pub struct FieldUri {
            pub id: Option<String>,
            pub value: Option<String>,
        }
        #[derive(Debug,Default,Json)]
        pub struct ArrayUri {
            pub id: Option<String>,
            pub values: Vec<Option<String>>,
        }
    });

    variant_stream.extend(quote! {
        String(FieldString),
        #[json(rename = "string[]")]
        ArrayString(ArrayString),
        #[json(rename = "Uri")]
        Uri(FieldUri),
        #[json(rename = "Uri[]")]
        ArrayUri(ArrayUri),
    });
    ref_variant_stream.extend(quote! {
        String(&'a FieldString),
        #[json(rename = "string[]")]
        ArrayString(&'a ArrayString),
    });
    impl_stream.extend(impl_from("string", false));

    type_stream.extend(quote! {
        #[derive(Debug,Json)]
        pub enum Member {
            Reference(Reference),
            List(SyncList),
            SyncObject(SyncObject),
            Enum(FieldEnum),
            Empty,
            #variant_stream
        }

        #impl_stream
    });

    let syntax: syn::File = syn::parse2(type_stream).unwrap();
    let output = prettyplease::unparse(&syntax);
    let filename = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("types.rs");
    std::fs::write(&filename, output).unwrap();
}