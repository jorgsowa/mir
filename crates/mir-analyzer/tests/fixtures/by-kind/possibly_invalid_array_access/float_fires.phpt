===description===
PossiblyInvalidArrayAccess fires when the union includes a float atom.
===file===
<?php
$a = rand(0, 1) > 0 ? 1.5 : ["hello"];
echo $a[0];
===expect===
PossiblyInvalidArrayAccess@3:5-3:10: Possibly invalid array access: '1.5|array{0: "hello"}' might not support []
