===description===
PossiblyInvalidArrayAccess fires when the union includes a bool atom.
===file===
<?php
$a = rand(0, 1) > 0 ? true : ["hello"];
echo $a[0];
===expect===
PossiblyInvalidArrayAccess@3:5-3:10: Possibly invalid array access: 'true|array{0: "hello"}' might not support []
