===description===
PossiblyInvalidArrayAccess fires when the union includes a bool atom.
===file===
<?php
$a = rand(0, 1) > 0 ? true : ["hello"];
echo $a[0];
//   ^^^^^ PossiblyInvalidArrayAccess: Possibly invalid array access: 'true|array{0: "hello"}' might not support []
