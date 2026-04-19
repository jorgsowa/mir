===source===
<?php
interface A {}
interface B {}
interface C {}

/** @param A&B&C $x */
function f($x): void {
    $_ = $x;
}
===expect===
