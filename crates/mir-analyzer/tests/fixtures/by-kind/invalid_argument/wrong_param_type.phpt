===description===
Wrong param type
===ignore===
TODO
===file===
<?php
$take_string = function(string $s): string { return $s; };
$take_string(42);
===expect===
