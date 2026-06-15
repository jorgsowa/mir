===description===
Unused param
===config===
suppress=MissingReturnType
===file===
<?php
function foo(int $i) {}

foo(4);
===expect===
UnusedParam@2:13-2:19: Parameter $i is never used
