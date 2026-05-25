===description===
Wrong param type
===file===
<?php
$take_string = function(string $s): string { return $s; };
$take_string(42);
===expect===
InvalidScalarArgument
===ignore===
TODO
