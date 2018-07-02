use std::mem;

pub fn clean_path(p: &str) -> String {
    if p == "" {
        return "/".to_string();
    }

    let n = p.len();
    let mut buf: Vec<u8> = Vec::new();
    let mut r = 1;
    let mut w = 1;

    if !p.starts_with("/") {
        r = 0;
        buf.resize(n + 1, 0);
        buf[0] = b'/';
    }

    let mut trailing = n > 1 && p.ends_with("/");
    let p = p.as_bytes();
    while r < n {
        match p[r] {
            b'/' => r += 1,
            b'.' => {
                if r + 1 == n {
                    trailing = true;
                    r += 1;
                } else if p[r + 1] == b'/' {
                    r += 1;
                } else if p[r + 1] == b'.' && (r + 2 == n || p[r + 2] == b'/') {
                    r += 2;

                    if w > 1 {
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
                if w > 1 {
                    buf_app(&mut buf, p, w, b'/');
                    w += 1;
                }

                while r < n && p[r] != b'/' {
                    buf_app(&mut buf, p, w, p[r]);
                    w += 1;
                    r += 1;
                }
            }
        }
    }
    if trailing && w > 1 {
        buf_app(&mut buf, p, w, b'/');
        w += 1;
    }

    if buf.is_empty() {
        return String::from_utf8(p[..w].to_vec()).unwrap();
    }
    String::from_utf8(buf[..w].to_vec()).unwrap()
}

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
