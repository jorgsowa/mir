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
InvalidArgument@15:9-15:16: Argument $iter of iterator() expects 'array<mixed, string>', got 'A'
