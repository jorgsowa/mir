===description===
A `@param` type with a genuinely unterminated string literal (an opening
quote with no closing quote, not just a lone quote character) must be
reported the same way as the lone-quote case.
===config===
suppress=UnusedParam
===file===
<?php

/**
 * @param 'foo $x
 */
function bar($x): void {}
===expect===
InvalidDocblock@3:0-3:0: Invalid docblock: @param has an unterminated string literal in `'foo`
MissingParamType@6:13-6:15: Parameter $x of bar() has no type annotation
