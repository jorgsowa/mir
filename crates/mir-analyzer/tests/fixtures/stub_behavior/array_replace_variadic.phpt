===description===
array_replace replacements are variadic and optional
===file===
<?php
// array_replace with just one base array (no replacements)
$result = array_replace(['x' => 1]);
// array_replace with one base array and one replacement
$result2 = array_replace(['x' => 1], ['y' => 2]);
// array_replace with multiple replacement arrays
$result3 = array_replace(['x' => 1], ['y' => 2], ['z' => 3]);
===expect===
