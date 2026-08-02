===description===
Variadic argument with no named arguments prevents passing array with string key
===file===
<?php
/**
 * @no-named-arguments
 * @return list<int>
 */
function foo(int ...$values): array
{
    return $values;
}

foo(...["a" => 0]);

===expect===
InvalidNamedArguments@11:4-11:17: foo() does not accept named arguments
