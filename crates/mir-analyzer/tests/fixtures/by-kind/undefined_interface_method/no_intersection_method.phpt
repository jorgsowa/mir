===description===
No intersection method
===ignore===
TODO
===file===
<?php
interface A {}
interface B {}

/** @param B&A $p */
function f($p): void {
    $p->zugzug();
}
===expect===
