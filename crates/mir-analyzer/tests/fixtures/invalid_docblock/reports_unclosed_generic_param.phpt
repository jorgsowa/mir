===file===
<?php
/**
 * @param array< $items
 */
function foo(mixed $items): void {}
===expect===
InvalidDocblock: Invalid docblock: @param has unclosed generic type `array< $items`
UnusedParam: Parameter $items is never used
