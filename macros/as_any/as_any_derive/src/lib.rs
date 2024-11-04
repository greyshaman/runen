use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(AsAny)]
pub fn as_any_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_as_any_derive(&ast)
}

fn impl_as_any_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl AsAny for #name {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    };

    gen.into()
}