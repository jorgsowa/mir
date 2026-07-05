===description===
A class-string argument that's a union of two literal strings (e.g. a
ternary) must have every branch validated against the codebase, not just
the first — the second, undefined branch here must still be caught.
===config===
suppress=UnusedParam
===file===
<?php
class RealClass {}

/** @param class-string $cls */
function take(string $cls): void {}

function test(bool $cond): void {
    take($cond ? 'RealClass' : 'TotallyBogusClassName');
}
===expect===
UndefinedClass@8:9-8:54: Class TotallyBogusClassName does not exist
