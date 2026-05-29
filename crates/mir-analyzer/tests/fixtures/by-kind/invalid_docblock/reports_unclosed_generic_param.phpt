===description===
reports unclosed generic param
===file===
<?php
/**
 * @param array< $items
 */
function foo(mixed $items): void {}
===expect===
InvalidDocblock@2:0-2:0: Invalid docblock: @param has unclosed generic type `array< $items`
UnusedParam@5:14-5:26: Parameter $items is never used
