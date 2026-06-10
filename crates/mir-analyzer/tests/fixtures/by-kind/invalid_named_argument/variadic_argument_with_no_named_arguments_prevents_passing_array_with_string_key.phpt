===description===
Variadic argument with no named arguments prevents passing array with string key
===ignore===
TODO
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
