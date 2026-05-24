===description===
reports deprecated static method
===file===
<?php
class Greeter {
    /** @deprecated use newGreet() instead */
    public static function oldGreet(string $name): void {}
}

function test(): void {
    Greeter::oldGreet('Alice');
}
===expect===
UnusedParam@4:37: Parameter $name is never used
DeprecatedMethodCall@8:5: Call to deprecated method Greeter::oldGreet: use newGreet() instead
