extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

#[proc_macro_derive(FromExcel)]
pub fn from_sheet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let name = &ast.ident;

  let fields = match ast.data {
    syn::Data::Struct(ref data) => fields_to_vec(&data.fields),
    _ => panic!(
      "Error implementing FromExcel Macro for {}- this should be derived from a struct",
      name
    ),
  };

  let gen = quote! {
    impl FromExcel for #name {
      fn from_excel(value: ExcelObject) -> #name {
        #name {#fields}
      }

      fn get_sheet_name() -> String {
        "#name".to_string()
      }
    }
  };
  // println! {"The Quote:\n\n{:#?}", gen};
  proc_macro::TokenStream::from(gen)
}

/// Turn the type into a string for inserting the recursive from_excel call
fn get_field_type(path: &syn::Type) -> proc_macro2::TokenStream {
  match path {
    syn::Type::Path(type_path) => {
      let segment = &type_path.path.segments.first();
      println!("First Segment:\n{:#?}", type_path);
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

fn fields_to_vec(fields: &syn::Fields) -> TokenStream {
  match fields {
    Fields::Named(ref fields) => {
      let iterator = fields.named.iter().map(|f| {
        let name = &f.ident;
        let field_type = get_field_type(&f.ty);
        quote! { #name: <#field_type>::from_excel(value) }
      });
      quote! { #(#iterator),* }
    }
    Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
  }
}
