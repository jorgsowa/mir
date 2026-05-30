===description===
Unused param
===file===
<?php
function foo(int $i) {}

foo(4);
===expect===
UnusedParam@2:14-2:20: Parameter $i is never used
