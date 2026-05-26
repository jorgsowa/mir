===description===
Invalid array offset
===file===
<?php
$x = ["a"];
$y = $x["b"];
===expect===
InvalidArrayOffset
===ignore===
TODO
