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

function test(): void {
    // Should accept any concrete type when T is template parameter
    identity(new User());
    identity("string");
    identity(123);
    identity(null);
}
===expect===
UnusedParam@8:19-8:31: Parameter $value is never used
