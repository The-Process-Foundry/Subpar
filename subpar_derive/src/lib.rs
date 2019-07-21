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
      fn from_excel(&self, value: ExcelObject) -> #name {
        #name {#fields}
      }
    }
  };
  // println! {"The Quote:\n\n{:#?}", gen};
  proc_macro::TokenStream::from(gen)
}

fn fields_to_vec(fields: &syn::Fields) -> TokenStream {
  match fields {
    Fields::Named(ref fields) => {
      let iterator = fields.named.iter().map(|f| {
        let name = &f.ident;
        let value = "Test Value";
        quote! { #name: #value.to_string() }
      });
      quote! { #(#iterator),* }
    }
    Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
  }
}
