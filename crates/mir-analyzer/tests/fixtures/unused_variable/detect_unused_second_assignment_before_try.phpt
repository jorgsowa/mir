===description===
detectUnusedSecondAssignmentBeforeTry
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
UnusedVariable
===ignore===
TODO
