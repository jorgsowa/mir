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
//       ^^^^^^^ InvalidArgument: Argument $iter of iterator() expects 'iterable<int|string, string>', got 'A'
===expect===
