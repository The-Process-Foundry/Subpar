extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

// // Build the return struct from the fields

// // Build the individual field values from the fields
// fn row_to_values(fields: &syn::Fields) -> TokenStream {
//   match fields {
//     Fields::Named(ref fields) => {
//       let iterator = fields.named.iter().map(|f| {
//         let name = &f.ident;
//         let field_type = get_field_type(&f.ty);
//         let quoted_name = id_to_lit(&f.ident);
//         match is_vec(&f.ty) {
//           Some(_) => {
//             println!("Skipping the vector");
//             quote! {}
//           }
//           None => quote! {
//             let #name = <#field_type>::from_excel(
//               get_cell(ExcelObject::Row(row.clone()), #quoted_name.to_string())
//                 .expect("Could not find payer column for Payment")
//               ).expect("Error parsing the column #quoted_name");
//           },
//         }
//       });
//       quote! { #(#iterator)* }
//     }
//     Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
//   }
// }

// fn wb_to_sheet(fields: &syn::Fields) -> TokenStream {
//   match fields {
//     Fields::Named(ref fields) => {
//       let iterator = fields.named.iter().map(|f| {
//         let name = &f.ident;
//         let quoted_name = id_to_lit(&f.ident);
//         match is_vec(&f.ty) {
//           Some(ident) => {
//             println!("I'm A Vec");
//             let vec_type = id_to_lit(&Some(ident));
//             quote! {
//               let #name = match wb.get_sheet(#quoted_name.to_string()) {
//                 Ok(sheet) => Vec::<#vec_type>::from_excel(ExcelObject::Sheet(sheet))
//                   .expect("Couldn't make the vector of #name"),
//                 Err(_) => panic!("Could not open the workbook for #name"),
//               };
//             }
//           }
//           None => {
//             println!("'{:#?}' does not appear to be a vector", name);
//             quote! {}
//           }
//         }
//       });
//       quote! { #(#iterator)* }
//     }
//     Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
//   }
// }

// fn build_row_block(ast: &syn::DeriveInput) -> TokenStream {
//   let name = &ast.ident;
//   let field_values = match ast.data {
//     syn::Data::Struct(ref data) => row_to_values(&data.fields),
//     _ => panic!(
//       "Error implementing FromExcel Macro for {}- this should be derived from a struct",
//       name
//     ),
//   };
//   let fields = match ast.data {
//     syn::Data::Struct(ref data) => fields_to_struct(&data.fields),
//     _ => panic!(
//       "Error implementing FromExcel Macro for {}- this should be derived from a struct",
//       name
//     ),
//   };
//   quote! {
//       ExcelObject::Row(row) => {
//         #field_values
//         Ok(
//           #name {
//             #fields
//           }
//         )
//       },
//   }
// }

fn fields_to_struct(fields: &syn::Fields) -> TokenStream {
  match fields {
    Fields::Named(ref fields) => {
      let iterator = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote! {
          #name: match #name {
            Ok(x) => x,
            Err(err) => panic!("Error processing payments:\n{:#?}", err),
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
      "Error implementing FromExcel Macro for {}- this should be derived from a struct",
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

fn id_to_lit(value: &Option<syn::Ident>) -> syn::LitStr {
  match value {
    None => panic!("row_to_values received a field with no name"),
    Some(value) => syn::LitStr::new(&format!("{}", value), proc_macro2::Span::call_site()),
  }
}

/// Turn the type into a string for inserting the recursive from_excel call
fn get_field_type(path: &syn::Type) -> proc_macro2::TokenStream {
  match path {
    syn::Type::Path(type_path) => {
      match &type_path.path.segments.len() {
        0 => panic!("I don't know what to do with a field type with no segments"),
        1 => {
          let segment = &type_path.path.segments.first();
          match segment {
            Some(syn::punctuated::Pair::Punctuated(seg, _token)) => {
              // println!("Segment:\n{:#?}", seg);
              quote! { #seg }
            }
            Some(syn::punctuated::Pair::End(seg)) => {
              // println!("Segment ENd:\n{:#?}", seg);
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
        _ => panic!("Received more than one type segment for TypePath"),
      }
    }
    _ => panic!("TypePath was not a path"),
  }
}

// Is this path a vector? If so, what is the name in the angle brackets
fn is_vec(path: &syn::Type) -> Option<syn::Ident> {
  match path {
    syn::Type::Path(type_path) => match &type_path.path.segments.len() {
      0 => panic!("is_vec received a path with no segments"),
      1 => match &type_path.path.segments.first() {
        Some(syn::punctuated::Pair::Punctuated(seg, _token)) => panic!(
          "Got a multi-segment type in is_vec macro, and we haven't implemented it yet:\n{:#?}",
          seg
        ),
        Some(syn::punctuated::Pair::End(seg)) => {
          println!("is_vec End:\n{:#?}", seg);
          None
        }
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
      "Error implementing FromExcel Macro for {}- this should be derived from a struct",
      name
    ),
  };

  match fields {
    Fields::Named(ref fields) => {
      let iterator = fields.named.iter().map(|f| {
        let name = &f.ident;
        let field_type = get_field_type(&f.ty);
        let quoted_name = id_to_lit(&f.ident);

        let vec_matches = match is_vec(&f.ty) {
          Some(vec_type) => {
            println!("Skipping the vector");
            quote! {
              ExcelObject::Row(_) => <#field_type>::from_excel(excel_object),
              ExcelObject::Workbook(wb) => {
                let sheet = match wb.get_sheet(#name.to_string()) {
                  Ok(sheet) => Vec::<#vec_type>::from_excel(&ExcelObject::Sheet(sheet))
                    .expect("Couldn't make the vector of {}", #quoted_name),
                  Err(_) => panic!("Could not open the workbook for  {}", #quoted_name),
                };
                <#field_type>::from_excel(sheet),y
              },
            }
          }
          // TODO: get_cell is only if it is a primitive. Need to enumerate the code so the user can overload
          //       specific data types for multi-column composites
          None => quote! {
            ExcelObject::Workbook(_) => <#field_type>::from_excel(excel_object),
            ExcelObject::Row(row) =>
             <#field_type>::from_excel(
              &get_cell(ExcelObject::Row(row.clone()), #quoted_name.to_string())
                .expect(&format!("Could not find payer column for {}", #quoted_name)[..])
              ),
          },
        };

        quote! {
          let #name = match excel_object {
            ExcelObject::Cell(_) => <#field_type>::from_excel(excel_object),
            ExcelObject::Sheet(_) => <#field_type>::from_excel(excel_object),
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
#[proc_macro_derive(FromExcel)]
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
    impl FromExcel for #name {
      fn from_excel(excel_object: &ExcelObject) -> Result<#name, SubparError> {
        #parse_funcs

        #return_obj
      }

      #obj_name_func
    }
  };

  proc_macro::TokenStream::from(gen)
}
