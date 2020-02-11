extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

#[derive(Clone, Debug)]
struct FieldOptions {
  /// Uses this value instead of the field value to figure out the sheet/column name
  rename: Option<String>,

  /// The name of a function that takes in an excel cell and returns a value of the given type or a
  /// SubparError
  parser: Option<String>,
}

fn parse_opts(attr: syn::Meta, opts: FieldOptions) -> FieldOptions {
  match attr {
    syn::Meta::List(meta) => {
      meta.nested.iter().fold(opts, |acc, param| {
        // println!("Current Value:\n{:#?}", param);
        match param {
          syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
            let value = match &name_value.lit {
              syn::Lit::Str(lit_str) => lit_str.value(),
              _ => panic!("Unhandled syn::Lit type"),
            };

            match name_value.path.segments[0]
              .ident
              .clone()
              .to_string()
              .as_ref()
            {
              "rename" => FieldOptions {
                rename: Some(value),
                ..acc
              },
              "parser" => FieldOptions {
                parser: Some(value),
                ..acc
              },
              x => panic!("parse_group error: '{:#?}' is an unknown option", x),
            }
          }
          _ => panic!("parse_group error: Unhandled syn::Meta in nested"),
        }
      })
    }
    _ => panic!("parse_group error: Unhandled syn::Meta type for at attribute root"),
  }
}
// TODO: Add in special: Row_Id
impl FieldOptions {
  pub fn default() -> FieldOptions {
    FieldOptions {
      rename: None,
      parser: None,
    }
  }

  pub fn load_attributes(attrs: &Vec<syn::Attribute>) -> FieldOptions {
    attrs.iter().fold(FieldOptions::default(), |acc, attr| {
      match attr.path.segments[0].ident == "subpar" {
        false => acc,
        true => match attr.parse_meta() {
          Err(err) => panic!("{}", err),
          Ok(meta) => parse_opts(meta, acc),
        },
      }
    })
  }
}

fn id_to_lit(value: &Option<syn::Ident>) -> syn::LitStr {
  match value {
    None => panic!("row_to_values received a field with no name"),
    Some(value) => syn::LitStr::new(&format!("{}", value), proc_macro2::Span::call_site()),
  }
}

fn fields_to_struct(fields: &syn::Fields) -> TokenStream {
  match fields {
    Fields::Named(ref fields) => {
      let iterator = fields.named.iter().map(|f| {
        let name = &f.ident;
        let lit_name = id_to_lit(name);
        quote! {
          #name: match #name {
            Ok(x) => x,
            Err(err) => panic!(format!("Error processing '{}':\n{:#?}", #lit_name, err)),
          }
        }
      });
      quote! { #(#iterator),* }
    }
    Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
  }
}

fn build_return_struct(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;

  let fields = match ast.data {
    syn::Data::Struct(ref data) => fields_to_struct(&data.fields),
    _ => panic!(
      "Error implementing SubparTable Macro for {}- this should be derived from a struct",
      name
    ),
  };

  quote! {
    Ok(
      #name {
        #fields
      }
    )
  }
}

/// Turn the type into a string for inserting the recursive from_excel call
fn get_field_type(path: &syn::Type) -> proc_macro2::TokenStream {
  match path {
    syn::Type::Path(type_path) => match &type_path.path.segments.len() {
      0 => panic!("I don't know what to do with a field type with no segments"),
      1 => {
        let segment = &type_path.path.segments.first();
        match segment {
          Some(seg) => quote! { #seg },
          None => panic!("Got None for the first identifier"),
        }
      }
      _ => panic!("Received more than one type segment for TypePath"),
    },
    _ => panic!("TypePath was not a path"),
  }
}

// Is this path a vector? If so, what is the name in the angle brackets
fn is_vec(path: &syn::Type) -> Option<TokenStream> {
  match path {
    syn::Type::Path(type_path) => match &type_path.path.segments.len() {
      0 => panic!("is_vec received a path with no segments"),
      1 => match &type_path.path.segments.first() {
        Some(seg) => match &seg.ident.to_string()[..] {
          "Vec" => match &seg.arguments {
            syn::PathArguments::AngleBracketed(angle_args) => {
              let args = &angle_args.args;
              let quoted = quote! { #args };
              Some(quoted)
            }
            x => panic!(
              "Unhandled Vector type. Macro received Segment arguments:\n{:#?}",
              x
            ),
          },
          _ => None,
        },
        None => None,
      },
      _ => panic!("Received more than one type segment for TypePath"),
    },
    _ => panic!("TypePath was not a path"),
  }
}

fn parse_funcs(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;

  // for each field:
  let fields = match ast.data {
    syn::Data::Struct(ref data) => &data.fields,
    _ => panic!(
      "Error implementing SubparTable Macro for {}- this should be derived from a struct",
      name
    ),
  };

  match fields {
    Fields::Named(ref fields) => {
      let iterator = fields.named.iter().map(|f| {
        let name = &f.ident;
        let field_type = get_field_type(&f.ty);

        let opts = FieldOptions::load_attributes(&f.attrs);
        let quoted_name = match &opts.rename {
          Some(name) => syn::LitStr::new(name.to_string().as_ref(), proc_macro2::Span::call_site()),
          None => id_to_lit(&f.ident),
        };

        let field_from_excel = match &opts.parser {
          Some(name) => {
            let func = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote! { #func }
          }
          None => quote! { <#field_type>::from_excel },
        };

        // println!("\n\n-->The Parsed Opts:\n{:#?}", opts);
        // println!("Quoted Name: {:#?}", quoted_name);
        // println!("Field from excel: {:#?}", field_from_excel);
        let vec_matches = match is_vec(&f.ty) {
          Some(vec_type) => {
            // println!("Looking at the quoted vec type: '{:#?}'", vec_type);
            quote! {
              ExcelObject::Row(_) => #field_from_excel(excel_object),
              ExcelObject::Workbook(wb) => {
                match wb.read_sheet(#quoted_name.to_string()) {
                  Ok(sheet) => Vec::<#vec_type>::from_excel(&ExcelObject::Sheet(sheet)),
                  Err(_) => panic!("Could not open the workbook for  {}", #quoted_name),
                }
              },
            }
          }
          // TODO: get_cell is only if it is a primitive. Need to enumerate the code so the user can overload
          //       specific data types for multi-column composites
          None => quote! {
            ExcelObject::Workbook(_) => #field_from_excel(excel_object),
            ExcelObject::Row(row) => {
              let cell = &subpar::get_cell(ExcelObject::Row(row.clone()), #quoted_name.to_string())
                .expect(&format!("Could not find column named '{}'", #quoted_name)[..]);
              #field_from_excel(cell)
            }
          },
        };

        quote! {
          let #name = match excel_object {
            ExcelObject::Cell(_) => #field_from_excel(excel_object),
            ExcelObject::Sheet(_) => #field_from_excel(excel_object),
            #vec_matches
          };
        }
      });
      quote! { #(#iterator)* }
    }
    Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
  }
}

// TODO: Look ahead? This doubles code since the branching is based upon children that we don't
//       know about yet
#[proc_macro_derive(SubparTable, attributes(subpar))]
pub fn from_sheet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let name = &ast.ident;
  let quoted_name = id_to_lit(&Some(name.clone()));

  let obj_name_func = quote! {
      fn get_object_name() -> String {
        #quoted_name.to_string()
      }
  };

  let parse_funcs = parse_funcs(&ast);
  let return_obj = build_return_struct(&ast);

  // loop worksheet - get row
  let gen = quote! {
    impl SubparTable for #name {
      fn from_excel(excel_object: &ExcelObject) -> Result<#name, SubparError> {
        #parse_funcs
        #return_obj
      }

      #obj_name_func
    }
  };

  proc_macro::TokenStream::from(gen)
}
