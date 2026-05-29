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
UnusedParam@4:37-4:49: Parameter $name is never used
DeprecatedMethodCall@8:5-8:31: Call to deprecated method Greeter::oldGreet: use newGreet() instead
