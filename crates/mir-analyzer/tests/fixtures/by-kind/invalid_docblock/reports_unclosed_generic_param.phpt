===description===
reports unclosed generic param
===file===
<?php
/**
 * @param array< $items
// ^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @param has unclosed generic type `array< $items`
 */
function foo(mixed $items): void {}
//           ^^^^^^^^^^^^ UnusedParam: Parameter $items is never used
