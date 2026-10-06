===description===
reports deprecated static method
===file===
<?php
class Greeter {
    /** @deprecated use newGreet() instead */
    public static function oldGreet(string $name): void {}
//                                  ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
}

function test(): void {
    Greeter::oldGreet('Alice');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedMethodCall: Call to deprecated method Greeter::oldGreet: use newGreet() instead
}
