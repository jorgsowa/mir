===description===
preventStringDocblockType
===file===
<?php
/**
 * @param string $mapper
 */
function map2(callable $mapper): void {}

map2("foo");
===expect===
MismatchingDocblockParamType
===ignore===
TODO
