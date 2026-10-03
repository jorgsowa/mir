===description===
Negative counterpart: no member declares the method nor `__call`, so
UndefinedMethod still fires, including inside a union with an intersection.
===config===
suppress=UnusedParam
===file===
<?php
interface A { public function a(): int; }
interface B { public function b(): string; }

function f(A&B $x): void {
    $x->nope();
//  ^^^^^^^^^^ UndefinedMethod: Method A&B::nope() does not exist
}
===expect===
