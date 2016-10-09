use std::fmt;
use std::error::Error as StdError;
use std::result;
// use regex crate to lex things, simpler, for now.

#[derive(Debug)]
pub enum ParseError {
    Syntax,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ParseError::Syntax => write!(f, "Syntax error"),
        }
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Syntax => "Syntax error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            ParseError::Syntax => None,
        }
    }
}


pub fn lex(str: String) -> Result<(u64, u64), ParseError> {
    // const char *ws = " \t\n";
    // const char *deli m = "() \t\n";
    // const char *prefix = "()";

    // str += strspn(str, ws);

    // if (str[0] == '\0') {
    //     *start = *end = NULL;
    //     return Error_Syntax;
    // }

    // *start = str;

    // if (strchr(prefix, str[0]) != NULL)
    //     *end = str + 1;
    // else
    //     *end = str + strcspn(str, delim);

    // return Error_OK;
    Result::Err(ParseError::Syntax)
}


int parse_simple(const char *start, const char *end, Atom *result)
{
    char *buf, *p;

    /* Is it an integer? */
    long val = strtol(start, &p, 10);
    if (p == end) {
	result->type = AtomType_Integer;
    result->value.integer = val;
return Error_OK;
	}

	/* NIL or symbol */
	buf = malloc(end - start + 1);
	p = buf;
	while (start != end)
		*p++ = toupper(*start), ++start;
	*p = '\0';

	if (strcmp(buf, "NIL") == 0)
		*result = nil;
	else
		*result = make_sym(buf);

	free(buf);

	return Error_OK;
}

// int read_list(const char *start, const char **end, Atom *result)
// {
//     Atom p;

//     *end = start;
//     p = *result = nil;

//     for (;;) {
// 	const char *token;
// 	Atom item;
// 	Error err;

// 	err = lex(*end, &token, end);
// 	if (err)
// 	    return err;

// 	if (token[0] == ')')
// 	return Error_OK;

//     if (token[0] == '.' && *end - token == 1) {
// 	/* Improper list */
// 	if (nilp(p))
// 	    return Error_Syntax;

// 	err = read_expr(*end, end, &item);
// 	if (err)
// 	    return err;

// 	cdr(p) = item;

// 	/* Read the closing ')' */
// 	err = lex(*end, &token, end);
// 	if (!err && token[0] != ')')
// 	err = Error_Syntax;

//     return err;
// }

// err = read_expr(token, end, &item);
// if (err)
//     return err;

// if (nilp(p)) {
//     /* First item */
//     *result = cons(item, nil);
//     p = *result;
// } else {
//     cdr(p) = cons(item, nil);
//     p = cdr(p);
// }
// 	}
// }

// int read_expr(const char *input, const char **end, Atom *result)
// {
//     const char *token;
//     Error err;

//     err = lex(input, &token, end);
//     if (err)
// 	return err;

//     if (token[0] == '(')
// 	return read_list(*end, end, result);
// 	else if (token[0] == ')')
// 	return Error_Syntax;
//     else
// 	return parse_simple(token, *end, result);
// }
