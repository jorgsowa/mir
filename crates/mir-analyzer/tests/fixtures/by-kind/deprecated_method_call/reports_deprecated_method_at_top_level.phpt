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
UnusedParam@4:30-4:42: Parameter $name is never used
DeprecatedMethod@8:1-8:22: Method Greeter::oldGreet() is deprecated: use newGreet() instead
