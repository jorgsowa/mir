===description===
A literal string containing '|' is parsed as one type, not split mid-literal
and collapsed to mixed — a mismatched argument is still checked.
===config===
suppress=UnusedParam
===file===
<?php
/** @param 'a|b'|'c' $x */
function g($x): void {}

g('z');
//^^^ InvalidArgument: Argument $x of g() expects '"a|b"|"c"', got '"z"'
===expect===
