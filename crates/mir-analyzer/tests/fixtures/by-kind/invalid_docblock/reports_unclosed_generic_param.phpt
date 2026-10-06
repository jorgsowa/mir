===description===
reports unclosed generic param
===file===
<?php
/**
 * @param array< $items
 */
function foo(mixed $items): void {}
//           ^^^^^^^^^^^^ UnusedParam: Parameter $items is never used
===expect===
InvalidDocblock@3:3-3:23: Invalid docblock: @param has unclosed generic type `array< $items`
