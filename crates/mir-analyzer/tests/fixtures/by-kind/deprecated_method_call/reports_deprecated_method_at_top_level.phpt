===description===
reports deprecated method at top level
===file===
<?php
class Greeter {
    /** @deprecated use newGreet() instead */
    public function oldGreet(string $name): void {}
//                           ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
}

$g = new Greeter();
$g->oldGreet('Alice');
//<^^^^^^^^^^^^^^^^^^^^^ DeprecatedMethod: Method Greeter::oldGreet() is deprecated: use newGreet() instead
