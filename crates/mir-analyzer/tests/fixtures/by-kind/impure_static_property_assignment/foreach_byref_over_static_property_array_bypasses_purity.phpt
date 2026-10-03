===description===
`foreach (Bag::$queue as &$v)` mutates a static property's array contents
in place, exactly as much as a by-ref call argument
(`array_push(Bag::$queue, 1)`, already checked) does — but the foreach
statement never routed the iterable expression through the same purity
check at all.
===config===
<mir>
  <issueHandlers>
    <UnusedForeachValue errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Bag {
    public static array $queue = [1, 2, 3];
}

/** @pure */
function bumpAll(): void {
    foreach (Bag::$queue as &$v) {
//           ^^^^^^^^^^^ ImpureStaticPropertyAssignment: Assigning to static property Bag::$queue in a @pure function
//                ^^^^^^ ImpureStaticPropertyAccess: Reading static property Bag::$queue in a @pure function
        $v++;
    }
}
===expect===
