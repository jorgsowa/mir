===file===
<?php
/**
 * @return array<
 */
function foo(): mixed { return []; }
===expect===
InvalidDocblock: Invalid docblock: @return has unclosed generic type `array<`
