===file===
<?php
class Greeter {
    /** @deprecated use newGreet() instead */
    public function oldGreet(string $name): void {}
}

$g = new Greeter();
$g->oldGreet('Alice');
===expect===
UnusedParam: Parameter $name is never used
DeprecatedMethodCall: Call to deprecated method Greeter::oldGreet: use newGreet() instead
