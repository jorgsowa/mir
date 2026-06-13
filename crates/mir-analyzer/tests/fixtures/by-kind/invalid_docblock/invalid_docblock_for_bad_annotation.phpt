===description===
Invalid docblock for bad annotation
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param-out array<a(),bool> $ar
 */
function foo(array &$ar) : void {}
===expect===
