===description===
A malformed literal-string union with an unbalanced quote count
("foo"with" has 3 quotes) is quote-aware-parsed rather than crashing. The
unbalanced quote is now reported as an unterminated string literal (see the
`unterminated_string_literal_*` fixtures in `invalid_docblock`), which
discards the bogus type entirely rather than InvalidArgument checking
`foo(4)` against a nonsensical pseudo-type parsed out of it.
===config===
suppress=UnusedParam
===file===
<?php
namespace Ns;

/** @param "foo"with"|"bar"|1|2|3 $s */
function foo($s) : void {}
foo(4);
===expect===
InvalidDocblock@4:0-4:0: Invalid docblock: @param has an unterminated string literal in `"foo"with"|"bar"|1|2|3`
MissingParamType@5:13-5:15: Parameter $s of foo() has no type annotation
