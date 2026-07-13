===description===
A valid member of a literal-string union containing '@' is not flagged.
===config===
suppress=UnusedParam
===file===
<?php
/** @param 'admin@example.com'|'guest@example.com' $email */
function f($email): void {}

f('admin@example.com');
===expect===
