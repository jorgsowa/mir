===description===
reports deprecated method at top level
===file===
<?php
class Greeter {
    /** @deprecated use newGreet() instead */
    public function oldGreet(string $name): void {}
}

$g = new Greeter();
$g->oldGreet('Alice');
===expect===
UnusedParam@4:29: Parameter $name is never used
DeprecatedMethodCall@8:0: Call to deprecated method Greeter::oldGreet: use newGreet() instead
