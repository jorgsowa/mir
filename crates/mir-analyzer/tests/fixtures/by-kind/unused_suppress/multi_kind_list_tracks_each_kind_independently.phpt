===description===
Negative control for the trailing-prose comma fix: a genuine comma-separated
multi-kind list must still register each kind as its own independent
suppression — one used, one not — rather than the whole list collapsing into
a single tracked kind.
===file===
<?php
class Foo {}
/**
 * @psalm-suppress UndefinedClass, UndefinedMethod
 */
function test(Foo $f): void {
    echo get_class($f);
    new NoSuchClass();
}
===expect===
UnusedSuppress@6:0-6:0: Suppress annotation for 'UndefinedMethod' is never used
