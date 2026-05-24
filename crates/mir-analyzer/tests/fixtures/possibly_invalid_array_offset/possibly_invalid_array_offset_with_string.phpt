===description===
possiblyInvalidArrayOffsetWithString
===file===
<?php
$x = rand(0, 5) > 2 ? ["a" => 5] : "hello";
$y = $x["a"];
===expect===
PossiblyInvalidArrayOffset
===ignore===
TODO
