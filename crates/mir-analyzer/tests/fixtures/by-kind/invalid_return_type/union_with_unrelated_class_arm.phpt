===description===
Returning a union where one class arm is unrelated to every declared class is invalid; narrowing a parent to a child stays lenient.
===file===
<?php
final class A {}
final class B {}
class ParentBox {}
class ChildBox extends ParentBox {}

function unguarded(A|B $x): B
{
    return $x;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'A|B' is not compatible with declared 'B'
}

function guarded(A|B $x): B
{
    if ($x instanceof A) {
        throw new LogicException('not B');
    }

    return $x;
}

function whole(A|B $x): A|B
{
    return $x;
}

function nullable(?A $x): ?A
{
    return $x;
}

function parentToChild(ParentBox $x): ChildBox
{
    return $x;
}
