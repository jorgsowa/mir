===description===
reports variable in return
===file===
<?php
/**
 * @return $bar
// ^^^^^^^^^^^^ InvalidDocblock: @return contains variable `$bar` in type position
 */
function foo(): mixed { return null; }
