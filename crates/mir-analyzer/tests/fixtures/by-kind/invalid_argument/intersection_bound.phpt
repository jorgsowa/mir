===description===
class implementing both parts of an intersection bound should satisfy the template param
===file===
<?php
interface Type {}

interface NamedType {}

class Both implements Type, NamedType {}

/**
 * @template T of Type&NamedType
 */
function f(Type&NamedType $t): void {
    echo (string) $t;
}

f(new Both());
===expect===
