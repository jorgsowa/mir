===description===
No error when accessing a generic open array (array<string, int>) — key set is unknown
===file===
<?php
/** @return array<string, int> */
function counts(): array { return ['a' => 1]; }

$c = counts();
echo $c['missing'];
===expect===
