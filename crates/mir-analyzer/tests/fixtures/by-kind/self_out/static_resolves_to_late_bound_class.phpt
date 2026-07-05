===description===
@psalm-self-out static resolves to the actual (late-bound) receiver class,
not the class that declares the method.
===config===
suppress=UnusedParam
===file===
<?php
class Base {
    /** @psalm-self-out static */
    public function touch(): void {}
}
class Sub extends Base {}

$s = new Sub();
$s->touch();
/** @mir-check $s is Sub */

===expect===
