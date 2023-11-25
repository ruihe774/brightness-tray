use syn::parse_macro_input;
use syn::Lit;
use quote::quote;

#[proc_macro]
#[allow(non_snake_case)]
pub fn L(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let l = parse_macro_input!(tokens as Lit);
    match l {
        Lit::Str(s) => {
            let s = s.value();
            let ws: Vec<_> = s.encode_utf16().collect();
            quote!([#(#ws),*])
        }
        Lit::Char(c) => {
            let c = c.value();
            let mut wc = [0; 1];
            c.encode_utf16(&mut wc);
            let wc = wc[0];
            quote!(#wc)
        }
        _ => panic!("invalid argument")
    }.into()
}
