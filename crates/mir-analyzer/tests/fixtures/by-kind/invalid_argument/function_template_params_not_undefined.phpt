===description===
simple function template parameter should not cause InvalidArgument errors
===file===
<?php
class User { }

/**
 * @template T
 * @param T $value
 */
function identity(mixed $value): void {}
//                ^^^^^^^^^^^^ UnusedParam: Parameter $value is never used

function test(): void {
    // Should accept any concrete type when T is template parameter
    identity(new User());
    identity("string");
    identity(123);
    identity(null);
}
