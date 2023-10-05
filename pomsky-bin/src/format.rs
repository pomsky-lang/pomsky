use std::io::Write;

macro_rules! efprintln {
    ($($args:tt)*) => {{
        $crate::format::efwriteln(helptext::text![$($args)*]);
    }};
}

macro_rules! efprint {
    ($($args:tt)*) => {{
        $crate::format::efwrite(helptext::text![$($args)*]);
    }};
}

pub(crate) fn supports_color() -> bool {
    matches!(
        ::supports_color::on_cached(::supports_color::Stream::Stderr),
        Some(::supports_color::ColorLevel { has_basic: true, .. })
    )
}

pub(crate) fn efwriteln(segments: &[helptext::Segment]) {
    let mut buf = std::io::stderr().lock();
    for segment in segments {
        let _ = segment.write(&mut buf, supports_color(), 0);
    }
    let _ = buf.write_all(b"\n");
}

pub(crate) fn efwrite(segments: &[helptext::Segment]) {
    let mut buf = std::io::stderr().lock();
    for segment in segments {
        let _ = segment.write(&mut buf, supports_color(), 0);
    }
}
