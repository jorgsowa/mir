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
UnusedParam@4:29-4:41: Parameter $name is never used
DeprecatedMethod@8:0-8:21: Method Greeter::oldGreet() is deprecated: use newGreet() instead
