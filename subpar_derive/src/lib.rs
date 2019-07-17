// #[macro_use]
// extern crate syn;
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

fn impl_from_row(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;
  let fields = match &ast.data {
    syn::Data::Struct(data) => &data.fields,
    _ => panic!("Error implementing FromExcel Macro for {}- this should be derived from a struct", name)
  };
  println!("The Fields:\n{:#?}", &fields);

  let gen = quote! {
      impl FromExcel for #name {
          fn from_excel_row(file_name: String) {
            #name {

            }
          }
      }
  };
  gen.into()
}

#[proc_macro_derive(FromExcel)]
pub fn from_sheet(input: TokenStream) -> TokenStream {
  let ast = syn::parse(input).unwrap();
  impl_from_row(&ast)
}
