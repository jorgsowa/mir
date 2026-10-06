===description===
A `@return` type with an unterminated string literal must be reported the
same way as `@var`/`@param`.
===config===
<mir/>
===file===
<?php

/**
 * @return 'foo
// ^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @return has an unterminated string literal in `'foo`
 */
function bar() {
    return 'foo';
}
===expect===
