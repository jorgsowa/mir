===description===
reports unclosed generic param
===file===
<?php
/**
 * @param array< $items
 */
function foo(mixed $items): void {}
===expect===
InvalidDocblock@2:0: Invalid docblock: @param has unclosed generic type `array< $items`
UnusedParam@5:13: Parameter $items is never used
===ignore===
TODO
