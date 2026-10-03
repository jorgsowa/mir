===description===
in_array($needle, [...]) without the strict 3rd argument must not narrow a
needle whose current type spans both strings and ints down to the
haystack's literal-int union — loose (==) comparison means the string "1"
matches the int 1, so $needle could still be a string here. The call stays
a `PossiblyInvalidArgument` (int|string is not assignable to string), not a
coerced `ArgumentTypeCoercion` down to `1|2`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(int|string $x): void {
    if (in_array($x, [1, 2])) {
        strlen($x);
//             ^^ PossiblyInvalidArgument: Argument $string of strlen() expects 'string', possibly different type 'int|string' provided
    }
}
===expect===
