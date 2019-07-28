extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

/// Turn the type into a string for inserting the recursive from_excel call
fn get_field_type(path: &syn::Type) -> proc_macro2::TokenStream {
  match path {
    syn::Type::Path(type_path) => {
      let segment = &type_path.path.segments.first();
      // println!("First Segment:\n{:#?}", type_path);
      match segment {
        Some(syn::punctuated::Pair::Punctuated(seg, _token)) => {
          quote! { #seg }
        }
        Some(syn::punctuated::Pair::End(seg)) => {
          quote! { #seg }
        }
        None => panic!("Got None for the first identifier"),
      }

      // .fold(Vec::new(), |mut acc, seg| {
      //   acc.push(seg.ident);
      //   match &seg.arguments {
      //     syn::PathArguments::None => println!("No Argument Here"),
      //     syn::PathArguments::Parenthesized(_args) => panic!("Got parenthized path arguments"),
      //     syn::PathArguments::AngleBracketed(args) => println!("got some args"),
      //   };
      //   acc
      // })
      // .first()
      // .unwrap(),
    }
    _ => panic!("No match on TypePath"),
  }
}

/// Use this to determine if we are looking at a leaf field
// fn field_value() -> bool {
//   false
// }

// Build the return struct from the fields
fn fields_to_struct(fields: &syn::Fields) -> TokenStream {
  match fields {
    Fields::Named(ref fields) => {
      let iterator = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: #name }
      });
      quote! { #(#iterator),* }
    }
    Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
  }
}

fn id_to_lit(value: &Option<syn::Ident>) -> syn::LitStr {
  match value {
    None => panic!("row_to_values received a field with no name"),
    Some(value) => syn::LitStr::new(&format!("{}", value), proc_macro2::Span::call_site()),
  }
}

// Build the individual field values from the fields
fn row_to_values(fields: &syn::Fields) -> TokenStream {
  match fields {
    Fields::Named(ref fields) => {
      let iterator = fields.named.iter().map(|f| {
        let name = &f.ident;
        let field_type = get_field_type(&f.ty);
        let quoted_name = id_to_lit(&f.ident);

        quote! {
          let #name = <#field_type>::from_excel(
            get_cell(ExcelObject::Row(row.clone()), #quoted_name.to_string())
              .expect("Could not find payer column for Payment")
            ).expect("Error parsing the column #quoted_name");
        }
      });
      quote! { #(#iterator)* }
    }
    Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
  }
}

#[proc_macro_derive(FromExcel)]
pub fn from_sheet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let name = &ast.ident;

  // let state = match ast.data {
  //     syn::Data::Struct(ref data) => fields_to_vec(&data.fields),
  //     _ => panic!(
  //         "Error implementing FromExcel Macro for {}- this should be derived from a struct",
  //         name
  //     ),
  // };

  let field_values = match ast.data {
    syn::Data::Struct(ref data) => row_to_values(&data.fields),
    _ => panic!(
      "Error implementing FromExcel Macro for {}- this should be derived from a struct",
      name
    ),
  };

  let fields = match ast.data {
    syn::Data::Struct(ref data) => fields_to_struct(&data.fields),
    _ => panic!(
      "Error implementing FromExcel Macro for {}- this should be derived from a struct",
      name
    ),
  };
  // println!("The fields:\n{:#?}", fields);
  // let action = quote! {
  //   println!("I'm the action function with workbook: {:#?}", wb);
  //   #name { #fields }
  // };
  // let sheet_func = open_sheet_func(name);
  // let wb_func = open_workbook_func(action);

  // open_wb
  // open

  // for each field,
  // If any primitive in the group, assume it is a leaf object

  let row_block = quote! {
    let row = match excel_object {
      ExcelObject::Row(row) => row,
      _ => panic!("#name FromExcel did not receive a row"),
    };
  };

  let obj_name_func = quote! {
      fn get_object_name() -> String {
        "#name".to_string()
      }
  };

  // loop worksheet - get row
  let gen = quote! {
    impl FromExcel for #name {
      fn from_excel(excel_object: ExcelObject) -> Result<#name, SubparError> {
        #row_block

        #field_values
        Ok(
          #name {
            #fields
          }
        )
      }

      #obj_name_func
    }
  };

  proc_macro::TokenStream::from(gen)
}
