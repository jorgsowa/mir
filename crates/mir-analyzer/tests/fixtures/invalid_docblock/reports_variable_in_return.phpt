===file===
<?php
/**
 * @return $bar
 */
function foo(): mixed { return null; }
===expect===
InvalidDocblock: Invalid docblock: @return contains variable `$bar` in type position
