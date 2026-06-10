===description===
Variadic argument is not list
===ignore===
TODO
===file===
<?php
/** @return list<int> */
function foo(int ...$values): array
{
    return $values;
}

===expect===
