use std::io::Write;

pub struct Io<W> {
    pub writer: W,
}

impl<W> Io<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> Write for Io<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl Default for Io<std::io::Stdout> {
    fn default() -> Self {
        Self {
            writer: std::io::stdout(),
        }
    }
}
