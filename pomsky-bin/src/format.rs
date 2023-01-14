use std::io::Write;

macro_rules! efprintln {
    ($($args:tt)*) => {{
        $crate::format::efwriteln(helptext::text![$($args)*]);
    }};
}

pub(crate) fn efwriteln(segments: &[helptext::Segment]) {
    let supports_color = matches!(
        supports_color::on_cached(supports_color::Stream::Stderr),
        Some(supports_color::ColorLevel { has_basic: true, .. })
    );

    let mut buf = std::io::stderr().lock();
    for segment in segments {
        let _ = segment.write(&mut buf, supports_color, 0);
    }
    let _ = buf.write_all(b"\n");
}
