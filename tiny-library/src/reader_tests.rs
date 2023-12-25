#[cfg(test)]
mod tests {
    use reader::{ReadError, Reader};
    use value::Value;

    #[test]
    fn test_read_empty() {
        let mut reader = Reader::new("_test_.tiny", "");
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn test_read_integers() {
        let mut reader = Reader::new(
            "_test_.tiny",
            "0 -0 +0 +1234 1234 -1234 +9223372036854775807 -9223372036854775808",
        );
        assert_eq!(reader.read(), Some(Ok(Value::Int(0))));
        assert_eq!(reader.read(), Some(Ok(Value::Int(0))));
        assert_eq!(reader.read(), Some(Ok(Value::Int(0))));
        assert_eq!(reader.read(), Some(Ok(Value::Int(1234))));
        assert_eq!(reader.read(), Some(Ok(Value::Int(1234))));
        assert_eq!(reader.read(), Some(Ok(Value::Int(-1234))));
        assert_eq!(reader.read(), Some(Ok(Value::Int(9223372036854775807))));
        assert_eq!(reader.read(), Some(Ok(Value::Int(-9223372036854775808))));
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn test_read_floats() {
        let mut reader = Reader::new("_test_.tiny", "0. 0.0 -0.0 +0.0 1.23 +1.23 -1.23 0.125");
        assert_eq!(reader.read(), Some(Ok(Value::Float(0.0))));
        assert_eq!(reader.read(), Some(Ok(Value::Float(0.0))));
        assert_eq!(reader.read(), Some(Ok(Value::Float(0.0))));
        assert_eq!(reader.read(), Some(Ok(Value::Float(0.0))));
        assert_eq!(reader.read(), Some(Ok(Value::Float(1.23))));
        assert_eq!(reader.read(), Some(Ok(Value::Float(1.23))));
        assert_eq!(reader.read(), Some(Ok(Value::Float(-1.23))));
        assert_eq!(reader.read(), Some(Ok(Value::Float(0.125))));
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn test_read_strings() {
        let mut reader = Reader::new(
            "_test_.tiny",
            r#"
"foo"
"bar"
"baz
quux"
"\t\r\n\\\""
"#,
        );
        assert_eq!(reader.read(), Some(Ok(Value::String("foo".into()))));
        assert_eq!(reader.read(), Some(Ok(Value::String("bar".into()))));
        assert_eq!(reader.read(), Some(Ok(Value::String("baz\nquux".into()))));
        assert_eq!(reader.read(), Some(Ok(Value::String("\t\r\n\\\"".into()))));
        assert_eq!(reader.read(), None);

        let mut reader = Reader::new("_test_.tiny", "\"foo\\x\"");
        assert_eq!(
            reader.read(),
            Some(Err(ReadError {
                name: reader.name.into(),
                start: 4,
                end: 6,
                message: "invalid string escape `\\x`".into(),
            }))
        );

        let mut reader = Reader::new("_test_.tiny", "   \"foo");
        assert_eq!(
            reader.read(),
            Some(Err(ReadError {
                name: reader.name.into(),
                start: 3,
                end: 7,
                message: "expected closing `\"`, found EOF".into(),
            }))
        );
    }

    #[test]
    fn test_read_symbols() {
        let mut reader = Reader::new(
            "_test_.tiny",
            r#"
foo
+foo
-foo
.foo
.*+!-_?$%&=<>:#123
+
-
namespaced/symbol
/
"#,
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "foo".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "+foo".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "-foo".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), ".foo".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), ".*+!-_?$%&=<>:#123".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "+".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "-".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "namespaced/symbol".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "/".into())))
        );
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn test_read_keywords() {
        let mut reader = Reader::new(
            "_test_.tiny",
            r#"
:foo
:+foo
:-foo
:.foo
:.*+!-_?$%&=<>:#123
:+
:-
:namespaced/keyword
:/
"#,
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), "foo".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), "+foo".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), "-foo".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), ".foo".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), ".*+!-_?$%&=<>:#123".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), "+".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), "-".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), "namespaced/keyword".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Keyword("".into(), "/".into())))
        );
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn test_read_commas() {
        let mut reader = Reader::new("_test_.tiny", ",, true ,false,");
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), ",,".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "true".into())))
        );
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), ",false,".into())))
        );
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn test_read_lists() {
        let mut reader = Reader::new(
            "_test_.tiny",
            "() (1 2 3) (true, false, nil) (((\"foo\" \"bar\")))",
        );

        assert_eq!(reader.read(), Some(Ok(Value::ListParen(vec![]))));

        assert_eq!(
            reader.read(),
            Some(Ok(Value::ListParen(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])))
        );

        assert_eq!(
            reader.read(),
            Some(Ok(Value::ListParen(vec![
                Value::Symbol("".into(), "true,".into()),
                Value::Symbol("".into(), "false,".into()),
                Value::Symbol("".into(), "nil".into()),
            ])))
        );

        assert_eq!(
            reader.read(),
            Some(Ok(Value::ListParen(vec![Value::ListParen(vec![
                Value::ListParen(vec![
                    Value::String("foo".into()),
                    Value::String("bar".into())
                ])
            ])])))
        );

        assert_eq!(reader.read(), None);

        let mut reader = Reader::new("_test_.tiny", "( (  1 2 3");
        assert_eq!(
            reader.read(),
            Some(Err(ReadError {
                name: reader.name.into(),
                start: 2,
                end: 10,
                message: "unclosed `(`".into(),
            }))
        );
    }

    #[test]
    fn test_read_vectors() {
        let mut reader = Reader::new(
            "_test_.tiny",
            "[] [1 2 3] [true, false, nil]
         [[[\"foo\" \"bar\"]]]",
        );

        assert_eq!(reader.read(), Some(Ok(Value::ListBracket(vec![]))));

        assert_eq!(
            reader.read(),
            Some(Ok(Value::ListBracket(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])))
        );

        assert_eq!(
            reader.read(),
            Some(Ok(Value::ListBracket(vec![
                Value::Symbol("".into(), "true,".into()),
                Value::Symbol("".into(), "false,".into()),
                Value::Symbol("".into(), "nil".into()),
            ])))
        );

        assert_eq!(
            reader.read(),
            Some(Ok(Value::ListBracket(vec![Value::ListBracket(vec![
                Value::ListBracket(vec![
                    Value::String("foo".into()),
                    Value::String("bar".into())
                ])
            ])])))
        );

        assert_eq!(reader.read(), None);

        let mut reader = Reader::new("_test_.tiny", "[ [  1 2 3");
        assert_eq!(
            reader.read(),
            Some(Err(ReadError {
                name: reader.name.into(),
                start: 2,
                end: 10,
                message: "unclosed `[`".into(),
            }))
        );
    }

    #[test]
    fn test_comments() {
        let mut reader = Reader::new(
            "_test_.tiny",
            "
        ; 0
        ;; ;
        0
        --;0
        +0
        [;[]
        ]
        {;}
        }
    ",
        );
        assert_eq!(reader.read(), Some(Ok(Value::Int(0))));
        assert_eq!(
            reader.read(),
            Some(Ok(Value::Symbol("".into(), "--".into())))
        );
        assert_eq!(reader.read(), Some(Ok(Value::Int(0))));
        assert_eq!(reader.read(), Some(Ok(Value::ListBracket(Vec::new()))));
        assert_eq!(reader.read(), Some(Ok(Value::ListBrace(Vec::new()))));
        assert_eq!(reader.read(), None);
    }
}
