/// The terminal input for a list of control codes and `term::Raw` text, in
/// order, like the `<=` list of `frag!`.
///
/// This lives apart from `frag!` so that tests which don't use `frag!` can
/// pull in just this file.
#[allow(unused_macros)]
macro_rules! input {
    ( $( $item:expr ),* $(,)? ) => {{
        #[allow(unused_imports)]
        use shpool_vterm::term::AsTermInput;
        let mut buf: Vec<u8> = vec![];
        $(
            $item.term_input_into(&mut buf);
        )*
        buf
    }};
}
