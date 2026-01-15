use std::fmt::Write;

use shpool_vterm::{term, ContentRegion};

macro_rules! frag {
    {
        $test_name:ident
        {
            scrollback_lines: $scrollback_lines:expr ,
            width: $width:expr ,
            height: $height:expr
        }
        <= $( $input_expr:expr ),*
        $( => $content_region:expr => $( $output_expr:expr ),* )*
    } => {
        #[test]
        fn $test_name() {
            use shpool_vterm::term::AsTermInput;
            let mut input: Vec<u8> = vec![];
            $(
                $input_expr.term_input_into(&mut input);
            )*

            $(
                {
                    let mut output: Vec<u8> = vec![];
                    $(
                        $output_expr.term_input_into(&mut output);
                    )*
                    crate::support::frag::round_trip_frag(
                        input.as_slice(), output.as_slice(),
                        $scrollback_lines,
                        shpool_vterm::Size{width: $width, height: $height},
                        $content_region);
                }
            )*
        }
    }
}

pub fn round_trip_frag(
    input: &[u8],
    want_output: &[u8],
    scrollback_lines: usize,
    size: shpool_vterm::Size,
    dump_content_region: ContentRegion,
) {
    let mut term = shpool_vterm::Term::new(scrollback_lines, size);
    term.process(input);
    eprintln!("\n\n================ ContentRegion: {dump_content_region:?}");
    let got_output = term.contents(dump_content_region);
    eprintln!("INPUT:\n{}\n---------------", PrettyTerm::pretty_print(input));
    eprintln!("WANT OUTPUT:\n{}\n--------------", PrettyTerm::pretty_print(want_output));
    eprintln!("GOT OUTPUT:\n{}\n--------------", PrettyTerm::pretty_print(got_output.as_slice()));
    eprintln!("TERM:\n{term}");
    if got_output == want_output {
        eprintln!("PASS");
    } else {
        eprintln!("FAIL");
    }
    assert_eq!(got_output, want_output);
}

// A pretty printer for a raw terminal input stream.
struct PrettyTerm {
    into: String,
}

impl PrettyTerm {
    fn pretty_print(input_stream: &[u8]) -> String {
        let mut parser = vte::Parser::new();
        let mut pretty_printer = PrettyTerm { into: String::new() };

        parser.advance(&mut pretty_printer, input_stream);
        pretty_printer.into
    }
}

impl vte::Perform for PrettyTerm {
    fn print(&mut self, c: char) {
        write!(self.into, "{}", c).unwrap();
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => write!(self.into, "<\\n>\n").unwrap(),
            b'\r' => write!(self.into, "<\\r>").unwrap(),
            _ => {
                eprintln!("pp: unhandled byte {}", byte);
            }
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // TODO: stub
    }

    fn put(&mut self, _byte: u8) {
        // TODO: stub
    }

    fn unhook(&mut self) {
        // TODO: stub
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // TODO: stub
    }

    // Handle escape codes beginning with the CSI indicator ('\x1b[').
    //
    // rustfmt has insane ideas about match arm formatting and there is
    // apparently no way to make it do the reasonable thing of preserving
    // horizontal whitespace by placing loops directly in match arm statement
    // position.
    #[rustfmt::skip]
    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        ignore: bool,
        action: char,
    ) {
        let code = term::ControlCode::CSI {
            params: params.iter()
                .map(|p| p.iter().map(|x| *x).collect::<Vec<u16>>())
                .collect::<Vec<Vec<u16>>>(),
            intermediates: intermediates.to_vec(),
            action,
        };
        if ignore {
            write!(self.into, "<ignored {}>", code).unwrap();
        } else {
            write!(self.into, "<{}>", code).unwrap();
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        let code = term::ControlCode::ESC { intermediates: intermediates.to_vec(), byte };
        if ignore {
            write!(self.into, "<ignored {}>", code).unwrap();
        } else {
            write!(self.into, "<{}>", code).unwrap();
        }
    }

    fn terminated(&self) -> bool {
        false
    }
}
