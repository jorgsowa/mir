===description===
Unused param
===file===
<?php
function foo(int $i) {}

foo(4);
===expect===
UnusedParam
===ignore===
TODO
