===description===
Unused list var
===file===
<?php
list($a, $b) = explode(" ", "hello world");
//       ^^ UnusedVariable: Variable $b is never read
echo $a;
