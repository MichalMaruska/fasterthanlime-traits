use lol_html::{element, HtmlRewriter, Settings, OutputSink};
use std::error::Error;
use html_escape::encode_safe_to_writer;
use std::io;

enum ProcessorType {
    LazyLoading,
    HtmlEscape,
}

impl ProcessorType {
    fn build<'w, W: io::Write>(&self, output: &'w mut W) -> Box<dyn Processor + 'w> {
        match self {
            ProcessorType::LazyLoading => Box::new(
                HtmlRewriter::new(
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
                    |c:&[u8]| output.write_all(c).unwrap(),
                )
            ),

            ProcessorType::HtmlEscape => Box::new(
                Escaper {
                    output: output,
                })
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let mut output = vec![];
    let input = include_str!("input.html");

    let rewriter = ProcessorType::LazyLoading.build(&mut output);
    process(input, rewriter)?;

    // Stream was ended twice.
    // rewriter.end()?;

    let output = std::str::from_utf8(&output).unwrap();
    println!("input: {input}");
    println!("output: {}", output);


    //
    let input = &output[..];
    let mut output = vec![];
    let escaper =  ProcessorType::HtmlEscape.build(&mut output);

    process(input, escaper)?;
    println!("output: escaped {}", std::str::from_utf8(&output[..]).unwrap());

    Ok(())
}

fn process(input: &str, mut processor: Box<dyn Processor + '_>) -> Result<(), Box<dyn Error>> {
    processor.write(input.as_bytes())?;
    processor.end()?;
    Ok(())
}



trait Processor {
    fn write(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>>;
    fn end(self: Box<Self>) -> Result<(), Box<dyn Error>>;
}


impl<'h,O: OutputSink> Processor for HtmlRewriter<'h, O> {

    fn write(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
        HtmlRewriter::write(self, chunk).map_err(Into::into)
    }

    fn end(self: Box<Self>) -> Result<(), Box<dyn Error>> {
        HtmlRewriter::end(*self).map_err(Into::into)
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

    fn end(self: Box<Self>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

}
