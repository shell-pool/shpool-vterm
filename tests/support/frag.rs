macro_rules! frag {
    {
        $test_name:ident
        {
            scrollback_lines: $scrollback_lines:expr ,
            width: $width:expr ,
            height: $height:expr
        }
        <= $( $input_expr:expr ),*
        => $( $output_expr:expr ),*
    } => {
        #[test]
        fn $test_name() {
            use shpool_vterm::term::AsTermInput;
            let mut input: Vec<u8> = vec![];
            $(
                $input_expr.term_input_into(&mut input);
            )*
            let mut output: Vec<u8> = vec![];
            $(
                $output_expr.term_input_into(&mut output);
            )*
            crate::support::frag::round_trip_frag(input.as_slice(), output.as_slice(),
                            $scrollback_lines,
                            shpool_vterm::Size{width: $width, height: $height});
        }
    }
}

pub fn round_trip_frag(
    input: &[u8],
    want_output: &[u8],
    scrollback_lines: usize,
    size: shpool_vterm::Size,
) {
    let mut term = shpool_vterm::Term::new(scrollback_lines, size);
    term.process(input);
    eprintln!("TERM:\n{term}");
    assert_eq!(term.contents().as_slice(), want_output);
}
