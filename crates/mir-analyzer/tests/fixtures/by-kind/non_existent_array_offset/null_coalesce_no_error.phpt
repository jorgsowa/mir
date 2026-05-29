===description===
No NonExistentArrayOffset on the LHS of ??
===file===
<?php
$a = ["k" => 1];
$x = $a["missing"] ?? "default";
===expect===
