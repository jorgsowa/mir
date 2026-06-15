===description===
Unused list var
===file===
<?php
list($a, $b) = explode(" ", "hello world");
echo $a;
===expect===
UnusedVariable@2:9-2:11: Variable $b is never read
