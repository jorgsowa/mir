===description===
possiblyInvalidArrayOffsetWithInt
===file===
<?php
$x = rand(0, 5) > 2 ? ["a" => 5] : "hello";
$y = $x[0];
===expect===
PossiblyInvalidArrayOffset
===ignore===
TODO
