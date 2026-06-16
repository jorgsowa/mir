===description===
Detect unused second assignment before try
===file===
<?php
$a = [1, 2, 3];
echo($a[0]);
$a = [4, 5, 6];

try {
  // something
} catch (Throwable $t) {
  // something else
}
===expect===
UnusedVariable@4:0-4:2: Variable $a is never read
