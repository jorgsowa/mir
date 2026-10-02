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
InvalidDocblock@2:0-2:0: Invalid docblock: @param has unclosed generic type `array< $items`
