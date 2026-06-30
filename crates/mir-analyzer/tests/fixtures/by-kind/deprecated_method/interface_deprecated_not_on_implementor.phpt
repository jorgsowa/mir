===description===
DeprecatedMethod does NOT fire when an interface declares a method as @deprecated but the implementing class provides a fresh, non-deprecated implementation — the deprecation is on the interface declaration, not the class method.
===config===
suppress=UnusedParam
===file===
<?php
interface Printable {
    /** @deprecated use render() instead */
    public function display(): void;
}
class Doc implements Printable {
    public function display(): void {}
}

function test(Doc $d): void {
    $d->display();
}
===expect===
