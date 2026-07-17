===description===
A `@return` type with an unterminated string literal must be reported the
same way as `@var`/`@param`.
===config===
===file===
<?php

/**
 * @return 'foo
 */
function bar() {
    return 'foo';
}
===expect===
InvalidDocblock@3:0-3:0: Invalid docblock: @return has an unterminated string literal in `'foo`
