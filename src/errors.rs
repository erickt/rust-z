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

    errors {
        HttpStatus(e: u32) {
            description("http request returned an unsuccessful status code")
            display("http request returned an unsuccessful status code: {}", e)
        }
        FileNotFound {
            description("file not found")
        }
    }
}

