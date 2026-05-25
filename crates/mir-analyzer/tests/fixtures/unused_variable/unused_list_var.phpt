===description===
Unused list var
===file===
<?php
list($a, $b) = explode(" ", "hello world");
echo $a;
===expect===
UnusedVariable
===ignore===
TODO
