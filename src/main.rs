use lol_html::{element, HtmlRewriter, Settings, OutputSink};
use std::error::Error;
use html_escape::encode_safe_to_writer;
use std::io;

enum ProcessorType {
    LazyLoading,
    HtmlEscape,
}

struct WriterOutputSink<W> {
    writer: W,
}

impl<W: io::Write> OutputSink for  WriterOutputSink<W> {
    fn handle_chunk(&mut self, chunk: &[u8]) {
        // might panic
        self.writer.write_all(chunk).unwrap()
    }
}

enum ProcessorImpl<W: io::Write> {
    LazyLoading(HtmlRewriter<'static, WriterOutputSink<W>>),
    HtmlEscape(Escaper<W>),
}

impl ProcessorType {
    fn build<'w, W: io::Write>(&self, output: &'w mut W) -> ProcessorImpl<W> {
        match self {
            ProcessorType::LazyLoading => ProcessorImpl::LazyLoading(
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
                    WriterOutputSink { writer: output},
                )
            ),

            ProcessorType::HtmlEscape => ProcessorImpl::HtmlEscape(
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

fn process<P: Processor>(input: &str, mut processor: P) -> Result<(), Box<dyn Error>> {
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

impl<W: io::Write> Processor for ProcessorImpl<W> {

    fn write(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
        match self {
            ProcessorImpl::LazyLoading(p) => p.write(chunk),
            ProcessorImpl::HtmlEscape(p) => p.write(chunk),
        }
    }

    fn end(self) -> Result<(), Box<dyn Error>> {
        match self {
            ProcessorImpl::LazyLoading(p) => p.end(),
            ProcessorImpl::HtmlEscape(p) => p.end(),
        }
    }
}
