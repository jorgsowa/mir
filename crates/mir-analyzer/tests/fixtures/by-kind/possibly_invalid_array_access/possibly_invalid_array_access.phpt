===description===
Possibly invalid array access
===file===
<?php
$a = rand(0, 10) > 5 ? 5 : ["hello"];
echo $a[0];
===expect===
PossiblyInvalidArrayAccess@3:5-3:10: Possibly invalid array access: '5|array{0: "hello"}' might not support []
