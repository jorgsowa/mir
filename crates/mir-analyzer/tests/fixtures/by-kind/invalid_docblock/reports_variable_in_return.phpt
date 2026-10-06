===description===
reports variable in return
===file===
<?php
/**
 * @return $bar
// ^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @return contains variable `$bar` in type position
 */
function foo(): mixed { return null; }
