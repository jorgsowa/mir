===description===
reports variable in return
===file===
<?php
/**
 * @return $bar
 */
function foo(): mixed { return null; }
===expect===
InvalidDocblock@3:3-3:15: Invalid docblock: @return contains variable `$bar` in type position
