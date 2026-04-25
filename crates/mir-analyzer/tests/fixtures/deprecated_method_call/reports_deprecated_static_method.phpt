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
UnusedParam: Parameter $name is never used
DeprecatedMethodCall: Call to deprecated method Greeter::oldGreet: use newGreet() instead
