===description===
A literal-string union containing '@' (e.g. email addresses) is parsed and
checked, not flagged as a malformed docblock type.
===config===
suppress=UnusedParam
===file===
<?php
/** @param 'admin@example.com'|'guest@example.com' $email */
function f($email): void {}

f('other@example.com');
===expect===
InvalidArgument@5:2-5:21: Argument $email of f() expects '"admin@example.com"|"guest@example.com"', got '"other@example.com"'
