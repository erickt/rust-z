error_chain! {
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links { }

    foreign_links {
        ::std::io::Error, Io,
        "temporary file error";
        ::yaml::ScanError, YamlScanError,
        "yaml scan error";
    }

    errors { }
}

