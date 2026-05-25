===description===
variadicArgumentIsNotList
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
