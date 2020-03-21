use super::frame::RawFrameBuffer;

struct Writer<'a> {
    raw: RawFrameBuffer<'a>,
    line: usize,
    col: usize,
}

impl<'a> Writer<'a> {
    pub fn new(frame: RawFrameBuffer<'a>) -> Self {
        Self {
            raw: frame,
            line: 0,
            col: 0,
        }
    }

    fn line_(&self) -> usize {
        self.line * 1024 * 20
    }

    pub fn write<T: Clone + Copy>(&mut self, value: T) {
        let next = self.line_() + self.col;
        assert!(next <= self.raw.max_size());
        for i in self.line_()..next {
            unsafe { self.raw.write_value(i, value) };
        }
        if self.col >= 1024 {
            self.col = 0;
        } else {
            self.col += 20;
        }
    }

    pub fn write_line<T: Clone + Copy>(&mut self, value: T) {
        self.col = 20;
        self.write(value);
        self.line += 1;
    }
}
