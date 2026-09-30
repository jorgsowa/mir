===description===
Mixed array offset
===file===
<?php
/** @var mixed */
$a = 5;
echo [1, 2, 3, 4][$a];
//                ^^ MixedArrayOffset: Mixed type used as array offset
===expect===
