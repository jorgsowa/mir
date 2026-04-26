===file===
<?php
function fill(int &$value): void { $value = 1; }
$n = 0;
fill($n);
echo $n;
===expect===
