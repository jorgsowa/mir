===description===
Unused conditional code
===file===
<?php
$a = 5;
if (rand(0, 1)) {
  $a = $a + 5;
}
===expect===
UnusedVariable@4:2-4:4: Variable $a is never read
