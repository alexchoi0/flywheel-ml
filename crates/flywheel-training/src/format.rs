use flywheel_ml_core::LabeledExample;

pub trait FormatWriter {
    fn write(&mut self, example: &LabeledExample) -> Result<(), std::io::Error>;
    fn flush(&mut self) -> Result<(), std::io::Error>;
}

pub struct JsonLinesWriter<W: std::io::Write> {
    writer: W,
}

impl<W: std::io::Write> JsonLinesWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: std::io::Write> FormatWriter for JsonLinesWriter<W> {
    fn write(&mut self, example: &LabeledExample) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(example)?;
        writeln!(self.writer, "{}", json)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.writer.flush()
    }
}
