/// CleanPath is the URL version of path.Clean, it returns a canonical URL path
/// for p, eliminating . and .. elements.
///
/// The following rules are applied iteratively until no further processing can
/// be done:
///	1. Replace multiple slashes with a single slash.
///	2. Eliminate each . path name element (the current directory).
///	3. Eliminate each inner .. path name element (the parent directory)
///	   along with the non-.. element that precedes it.
///	4. Eliminate .. elements that begin a rooted path:
///	   that is, replace "/.." by "/" at the beginning of a path.
///
/// If the result of this process is an empty string, "/" is returned
pub fn clean_path(p: &str) -> String {
    // Turn empty string into "/"
    if p == "" {
        return "/".to_string();
    }

    let n = p.len();
    let mut buf: Vec<u8> = Vec::new();

    // Invariants:
	//      reading from path; r is index of next byte to process.
	//      writing to buf; w is index of next byte to write.

	// path must start with '/'

    let mut r = 1;
    let mut w = 1;

    if !p.starts_with("/") {
        r = 0;
        buf.resize(n + 1, 0);
        buf[0] = b'/';
    }

    let mut trailing = n > 1 && p.ends_with("/");
    let p = p.as_bytes();

    // A bit more clunky without a 'lazybuf' like the path package, but the loop
	// gets completely inlined (bufApp). So in contrast to the path package this
	// loop has no expensive function calls (except 1x make)

    while r < n {
        match p[r] {
            b'/' => r += 1,  // empty path element, trailing slash is added after the end
            b'.' => {
                if r + 1 == n {
                    trailing = true;
                    r += 1;
                } else if p[r + 1] == b'/' {
                    // . element
                    r += 2;
                } else if p[r + 1] == b'.' && (r + 2 == n || p[r + 2] == b'/') {
                    // .. element: remove to last /
                    r += 3;

                    if w > 1 {
                        // can backtrack
                        w -= 1;

                        if buf.is_empty() {
                            while w > 1 && p[w] != b'/' {
                                w -= 1;
                            }
                        } else {
                            while w > 1 && buf[w] != b'/' {
                                w -= 1;
                            }
                        }
                    }
                }
            }
            _ => {
                // real path element.
			    // add slash if needed
                if w > 1 {
                    buf_app(&mut buf, p, w, b'/');
                    w += 1;
                }

                // copy element
                while r < n && p[r] != b'/' {
                    buf_app(&mut buf, p, w, p[r]);
                    w += 1;
                    r += 1;
                }
            }
        }
    }

    // re-append trailing slash
    if trailing && w > 1 {
        buf_app(&mut buf, p, w, b'/');
        w += 1;
    }

    if buf.is_empty() {
        return String::from_utf8(p[..w].to_vec()).unwrap();
    }
    String::from_utf8(buf[..w].to_vec()).unwrap()
}

/// internal helper to lazily create a buffer if necessary
fn buf_app(buf: &mut Vec<u8>, s: &[u8], w: usize, c: u8) {
    if buf.is_empty() {
        if s[w] == c {
            return;
        }
        buf.resize(s.len(), 0);
        buf[..w].copy_from_slice(&s[..w]);
    }
    buf[w] = c;
}

#[cfg(test)]
mod tests {
    use super::*;

    // path, result
    fn clean_tests() -> Vec<(&'static str, &'static str)> {
        vec![
            // Already clean
            ("/", "/"),
            ("/abc", "/abc"),
            ("/a/b/c", "/a/b/c"),
            ("/abc/", "/abc/"),
            ("/a/b/c/", "/a/b/c/"),
            // missing root
            ("", "/"),
            ("a/", "/a/"),
            ("abc", "/abc"),
            ("abc/def", "/abc/def"),
            ("a/b/c", "/a/b/c"),
            // Remove doubled slash
            ("//", "/"),
            ("/abc//", "/abc/"),
            ("/abc/def//", "/abc/def/"),
            ("/a/b/c//", "/a/b/c/"),
            ("/abc//def//ghi", "/abc/def/ghi"),
            ("//abc", "/abc"),
            ("///abc", "/abc"),
            ("//abc//", "/abc/"),
            // Remove . elements
            (".", "/"),
            ("./", "/"),
            ("/abc/./def", "/abc/def"),
            ("/./abc/def", "/abc/def"),
            ("/abc/.", "/abc/"),
            // Remove .. elements
            ("..", "/"),
            ("../", "/"),
            ("../../", "/"),
            ("../..", "/"),
            ("../../abc", "/abc"),
            ("/abc/def/ghi/../jkl", "/abc/def/jkl"),
            ("/abc/def/../ghi/../jkl", "/abc/jkl"),
            ("/abc/def/..", "/abc"),
            ("/abc/def/../..", "/"),
            ("/abc/def/../../..", "/"),
            ("/abc/def/../../..", "/"),
            ("/abc/def/../../../ghi/jkl/../../../mno", "/mno"),
            // Combinations
            ("abc/./../def", "/def"),
            ("abc//./../def", "/def"),
            ("abc/../../././../def", "/def"),
        ]
    }

    #[test]
    fn test_path_clean() {
        let tests = clean_tests();
        for test in tests {
            let s = clean_path(test.0);
            assert_eq!(test.1, s);

            let s = clean_path(test.1);
            assert_eq!(test.1, s);
        }
    }

    // #[test]
    // fn test_path_clean_mallocs() {

    // }
}
