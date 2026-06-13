===description===
Invalid iterable arg
===config===
suppress=UnusedForeachValue
===file===
<?php
/**
 * @param  iterable<string> $iter
 */
function iterator(iterable $iter): void
{
    foreach ($iter as $val) {
        //
    }
}

class A {
}

iterator(new A());
===expect===
InvalidArgument@15:10-15:17: Argument $iter of iterator() expects 'array<mixed, string>', got 'A'
