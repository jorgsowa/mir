===description===
Possibly invalid array access
===ignore===
TODO
===file===
<?php
$a = rand(0, 10) > 5 ? 5 : ["hello"];
echo $a[0];
===expect===
