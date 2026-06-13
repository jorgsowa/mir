===description===
No NonExistentArrayOffset on the LHS of ??
===config===
suppress=UnusedVariable
===file===
<?php
$a = ["k" => 1];
$x = $a["missing"] ?? "default";
===expect===
