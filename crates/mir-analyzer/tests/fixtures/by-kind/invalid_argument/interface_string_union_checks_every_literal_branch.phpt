===description===
An interface-string argument that's a union of two literal strings (e.g. a
ternary) must have every branch validated, not just the first — the second,
non-interface branch here must still be caught as NotAnInterface even though
the first branch is a valid interface name.
===config===
suppress=UnusedParam
===file===
<?php
interface Shape {}
class ConcreteThing {}

/** @param interface-string $ifaceName */
function describe(string $ifaceName): void {}

function test(bool $cond): void {
    describe($cond ? 'Shape' : 'ConcreteThing');
}
===expect===
NotAnInterface@9:13-9:46: ConcreteThing is not an interface
