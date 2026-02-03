use lol_html::{element, HtmlRewriter, Settings, OutputSink};
use std::error::Error;


fn main() -> Result<(), Box<dyn Error>> {
    let mut output = vec![];
    let input = include_str!("input.html");

    let mut rewriter = HtmlRewriter::try_new(
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
    )?;
    process(input, &mut rewriter)?;

    // rewriter.end()?;

    println!("input: {input}");
    println!("output: {}", std::str::from_utf8(&output).unwrap());
    Ok(())
}

fn process(input: &str, processor: &mut dyn Processor) -> Result<(), Box<dyn Error>> {
    processor.write(input.as_bytes())?;
    processor.end()?;

    Ok(())
}



trait Processor {
    fn write(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>>;
    fn end(&mut self) -> Result<(), Box<dyn Error>>;
}


impl<'h,O: OutputSink> Processor for HtmlRewriter<'h, O> {

    fn write(&mut self, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
        HtmlRewriter::write(self, chunk).map_err(Into::into)
    }

    fn end(&mut self) -> Result<(), Box<dyn Error>> {
        HtmlRewriter::end(self).map_err(Into::into)
    }
}
