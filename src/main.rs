use lol_html::{element, HtmlRewriter, Settings, OutputSink};
use std::error::Error;
use html_escape::encode_safe_to_writer;
use std::io;


fn main() -> Result<(), Box<dyn Error>> {
    let mut output = vec![];
    let input = include_str!("input.html");

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers:
            vec![
                element!("img", |el| {
                    el.set_attribute("loading", "lazy")?;
                    Ok(())
                })
            ],
            ..Default::default()
        },
        |c:&[u8]| output.extend_from_slice(c)
    );
    process(input, &mut rewriter)?;

    // Stream was ended twice.
    // rewriter.end()?;

    let output = std::str::from_utf8(&output).unwrap();
    println!("input: {input}");
    println!("output: {}", output);


    //
    let input = &output[..];
    let mut output = vec![];
    let mut escaper = Escaper {
        output: &mut output,
    };

    process(input, &mut escaper);
    println!("output: escaped {}", std::str::from_utf8(&output[..]).unwrap());

    Ok(())
}

fn process(input: &str, processor: &mut dyn Processor) -> Result<(), Box<dyn Error>> {
    processor.write(input.as_bytes())?;
    processor.end()?;
    Ok(())
}



trait Processor {
    fn write(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>>;
    fn end(self) -> Result<(), Box<dyn Error>>;
}


impl<'h,O: OutputSink> Processor for HtmlRewriter<'h, O> {

    fn write(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
        HtmlRewriter::write(self, chunk).map_err(Into::into)
    }

    fn end(self) -> Result<(), Box<dyn Error>> {
        HtmlRewriter::end(self).map_err(Into::into)
    }
}


struct Escaper<W: io::Write> {
    output: W,
}

impl<W: io::Write> Processor for Escaper<W> {

    fn write(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
        // fixme: chunk might end inside multibyte, so no UTF8
        encode_safe_to_writer(std::str::from_utf8(chunk)?,
                              &mut self.output).map_err(Into::into)
    }

    fn end(self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

}
