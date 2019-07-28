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

fn fields_to_vec(fields: &syn::Fields) -> TokenStream {
    match fields {
        Fields::Named(ref fields) => {
            let iterator = fields.named.iter().map(|f| {
                let name = &f.ident;
                let field_type = get_field_type(&f.ty);
                let value = quote! { <#field_type>::from_excel(ExcelObject::Workbook(wb)) };
                quote! { #name: #value }
            });
            quote! { #(#iterator),* }
        }
        Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
    }
}

// Function to open the workbook
fn open_workbook_func(action: TokenStream) -> TokenStream {
    quote! {
        let wb: calamine::Xlsx<_> =
          calamine::open_workbook(path).expect("Couldn't open the xlsx file");
        let wb_obj = Box::new(wb);
    }
}

// fn open_sheet_func(sheet_name: &proc_macro2::Ident) -> TokenStream {
//     quote!{
//       if let Some(Ok(range)) = xls.worksheet_range(#sheet_name) {
//         let (height, width) = range.get_size();
//             println!("Rows: {} x {} ", height, width);

//             let mut rows = range.rows();

//         } else {
//             println!("Couldn't find sheet")
//         }
//     }
// }

// fn vec_processor(input: TokenStream) -> TokenStream {
//   quote! {println!("vector input:\n{:#?}", input);}
// }

// fn struct_processor(input: TokenStream) -> TokenStream {
//   quote! {println!("struct input:\n{:#?}", input);}
// }

#[proc_macro_derive(FromExcel)]
pub fn from_sheet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let state = match ast.data {
        syn::Data::Struct(ref data) => fields_to_vec(&data.fields),
        _ => panic!(
            "Error implementing FromExcel Macro for {}- this should be derived from a struct",
            name
        ),
    };

    let _fields = match ast.data {
        syn::Data::Struct(ref data) => fields_to_vec(&data.fields),
        _ => panic!(
            "Error implementing FromExcel Macro for {}- this should be derived from a struct",
            name
        ),
    };
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

    // loop worksheet - get row
    let gen = quote! {
      impl FromExcel for #name {
        fn from_excel(excel_object: ExcelObject) -> #name {
          # state
        }

        fn get_object_name() -> String {
          "#name".to_string()
        }
      }
    };
    // println! {"The Quote:\n\n{:#?}", gen};
    proc_macro::TokenStream::from(gen)
}
