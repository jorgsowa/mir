===description===
A method missing from every part of an intersection type is UndefinedMethod.
===file===
<?php
interface A {}
interface B {}

/** @param B&A $p */
function f($p): void {
    $p->zugzug();
}
===expect===
UndefinedMethod@7:4-7:16: Method B&A::zugzug() does not exist
