===description===
array_merge arrays are variadic
===file===
<?php
// array_merge with one array
$merged = array_merge(['x' => 1]);
// array_merge with multiple arrays
$merged2 = array_merge(['x' => 1], ['y' => 2]);
// array_merge with more arrays
$merged3 = array_merge(['x' => 1], ['y' => 2], ['z' => 3]);
===expect===
