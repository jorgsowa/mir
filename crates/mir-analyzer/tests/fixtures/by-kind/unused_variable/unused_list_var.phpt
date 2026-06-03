===description===
Unused list var
===file===
<?php
list($a, $b) = explode(" ", "hello world");
echo $a;
===expect===
PossiblyInvalidArrayOffset@2:1-2:43: Array offset might be invalid: expects 'array', got 'array<int, string>|false'
UnusedVariable@2:10-2:12: Variable $b is never read
