===description===
Variadic argument is not list
===file===
<?php
/** @return list<int> */
function foo(int ...$values): array
{
    return $values;
}

===expect===
LessSpecificReturnStatement
===ignore===
TODO
