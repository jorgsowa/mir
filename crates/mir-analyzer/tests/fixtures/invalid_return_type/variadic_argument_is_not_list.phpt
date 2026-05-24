===description===
variadicArgumentIsNotList
===file===
<?php
/** @psalm-return list<int> */
function foo(int ...$values): array
{
    return $values;
}

===expect===
LessSpecificReturnStatement
===ignore===
TODO
