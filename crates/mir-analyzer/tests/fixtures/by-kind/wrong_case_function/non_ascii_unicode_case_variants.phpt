===description===
PHP function lookup is ASCII-case-insensitive only. fñoo and fÑoo differ in
non-ASCII bytes (ñ ≠ Ñ), so the exact name is not found; the result is
UndefinedFunction, not WrongCaseFunction. The exact-match spelling is not reported.
===file===
<?php
function fñoo(): void {}
fñoo();
fÑoo();
//<^^^^^^ UndefinedFunction: Function fÑoo() is not defined
