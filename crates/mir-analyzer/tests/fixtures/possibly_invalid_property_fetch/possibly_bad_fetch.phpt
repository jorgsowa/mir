===description===
possiblyBadFetch
===file===
<?php
$a = rand(0, 5) > 3 ? "hello" : new stdClass;
echo $a->foo;
===expect===
PossiblyInvalidPropertyFetch
===ignore===
TODO
