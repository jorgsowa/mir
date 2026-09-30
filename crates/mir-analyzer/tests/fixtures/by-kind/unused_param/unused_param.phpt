===description===
Unused param
===config===
suppress=MissingReturnType
===file===
<?php
function foo(int $i) {}
//           ^^^^^^ UnusedParam: Parameter $i is never used

foo(4);
===expect===
