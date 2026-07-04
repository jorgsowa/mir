===description===
A closure parameter with a default value (e.g. the common legacy
`int $a = null` implicit-nullable pattern) is skipped by the parameter-type
check entirely — its declared type doesn't reflect what it actually accepts at
the call boundary, so comparing it directly would produce a false positive.
===file===
<?php
/** @param callable(?int):void $c */
function process(callable $c): void {
    $c(null);
}
process(function (int $a = null): void {});
===expect===
