===description===
A method missing from every part of an intersection type is UndefinedMethod.
===file===
<?php
interface A {}
interface B {}

/** @param B&A $p */
function f($p): void {
    $p->zugzug();
//  ^^^^^^^^^^^^ UndefinedMethod: Method B&A::zugzug() does not exist
}
===expect===
