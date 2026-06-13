===description===
Unused list var
===file===
<?php
list($a, $b) = explode(" ", "hello world");
echo $a;
===expect===
UnusedVariable@2:10-2:12: Variable $b is never read
