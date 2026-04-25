===file===
<?php
// null callback (zip mode) is valid PHP 8 — callable|null signature
$result = array_map(null, [1, 2], [3, 4]);
===expect===
