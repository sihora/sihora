use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// Derive the `Model` trait for a struct.
///
/// The struct must have named fields. The first field must be `id: i64`,
/// which becomes the primary key. Supported field types: `i64`, `f64`,
/// `String`, `bool`.
///
/// # Example
/// ```ignore
/// #[derive(Clone, Model)]
/// struct Todo {
///     id: i64,
///     title: String,
///     done: bool,
/// }
/// ```
#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "Model can only be derived for structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(&input, "Model requires named fields")
            .to_compile_error()
            .into();
    };

    let table_name = format!("{}s", to_snake_case(&name.to_string()));

    let mut col_defs = Vec::new();
    let mut to_params = Vec::new();
    let mut from_fields = Vec::new();

    for (i, field) in fields.named.iter().enumerate() {
        let fname = field.ident.as_ref().unwrap();
        let is_id = fname == "id";
        let col_ty = type_to_column_type(&field.ty);

        col_defs.push(quote! {
            sihora::data::ColumnDef {
                name: stringify!(#fname),
                ty: sihora::data::#col_ty,
                primary_key: #is_id,
            }
        });

        if !is_id {
            let val = value_expr(fname, &field.ty);
            to_params.push(val);
        }

        let from_expr = from_value_expr(fname, &field.ty, i);
        from_fields.push(from_expr);
    }

    let num_cols = col_defs.len();

    let expanded = quote! {
        impl sihora::data::Model for #name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [sihora::data::ColumnDef] {
                static COLS: [sihora::data::ColumnDef; #num_cols] = [#(#col_defs),*];
                &COLS
            }

            fn id(&self) -> i64 {
                self.id
            }

            fn set_id(&mut self, id: i64) {
                self.id = id;
            }

            fn to_params(&self) -> Vec<sihora::data::Value> {
                vec![#(#to_params),*]
            }

            fn from_values(values: &[sihora::data::Value]) -> Self {
                Self {
                    #(#from_fields),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn type_to_column_type(ty: &Type) -> proc_macro2::TokenStream {
    let ty_str = quote!(#ty).to_string().replace(' ', "");
    match ty_str.as_str() {
        "i64" => quote!(ColumnType::Integer),
        "f64" => quote!(ColumnType::Real),
        "String" => quote!(ColumnType::Text),
        "bool" => quote!(ColumnType::Bool),
        other => {
            let msg = format!("unsupported Model field type: {other}");
            quote!(compile_error!(#msg))
        }
    }
}

fn value_expr(fname: &syn::Ident, ty: &Type) -> proc_macro2::TokenStream {
    let ty_str = quote!(#ty).to_string().replace(' ', "");
    match ty_str.as_str() {
        "i64" => quote!(sihora::data::Value::Integer(self.#fname)),
        "f64" => quote!(sihora::data::Value::Real(self.#fname)),
        "String" => quote!(sihora::data::Value::Text(self.#fname.clone())),
        "bool" => quote!(sihora::data::Value::Bool(self.#fname)),
        _ => quote!(compile_error!("unsupported type")),
    }
}

fn from_value_expr(fname: &syn::Ident, ty: &Type, idx: usize) -> proc_macro2::TokenStream {
    let ty_str = quote!(#ty).to_string().replace(' ', "");
    let extract = match ty_str.as_str() {
        "i64" => quote! {
            match &values[#idx] { sihora::data::Value::Integer(v) => *v, _ => 0 }
        },
        "f64" => quote! {
            match &values[#idx] { sihora::data::Value::Real(v) => *v, _ => 0.0 }
        },
        "String" => quote! {
            match &values[#idx] { sihora::data::Value::Text(v) => v.clone(), _ => String::new() }
        },
        "bool" => quote! {
            match &values[#idx] { sihora::data::Value::Bool(v) => *v, _ => false }
        },
        _ => quote!(compile_error!("unsupported type")),
    };
    quote!(#fname: #extract)
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}
