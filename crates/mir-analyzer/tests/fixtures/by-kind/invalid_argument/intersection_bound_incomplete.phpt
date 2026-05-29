===description===
class implementing only some parts of an intersection bound should fail
===file===
<?php
interface Type {}
interface NamedType {}

class PartialImpl implements Type {}

/**
 * @template T of Type&NamedType
 */
function f(Type&NamedType $t): void {
    echo (string) $t;
}

f(new PartialImpl());
===expect===
InvalidArgument@14:3-14:20: Argument $t of f() expects 'Type&NamedType', got 'PartialImpl'
