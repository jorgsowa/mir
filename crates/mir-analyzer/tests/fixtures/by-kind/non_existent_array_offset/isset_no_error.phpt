===description===
No NonExistentArrayOffset inside isset()
===file===
<?php
$a = ["k" => 1];
isset($a["missing"]);
===expect===
