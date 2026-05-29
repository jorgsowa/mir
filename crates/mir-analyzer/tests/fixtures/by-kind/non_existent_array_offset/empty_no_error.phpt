===description===
No NonExistentArrayOffset inside empty()
===file===
<?php
$a = ["k" => 1];
empty($a["missing"]);
===expect===
