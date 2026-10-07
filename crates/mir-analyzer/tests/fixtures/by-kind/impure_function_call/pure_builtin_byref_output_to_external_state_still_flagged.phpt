===description===
A pure builtin's by-ref output written to a property or by-ref parameter is
still an external mutation, reported on the argument instead of the call.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Matcher {
    /** @var array<array-key, string> */
    public array $groups = [];
}

/** @pure */
function capture(Matcher $m, string $subject, ?array &$out): bool {
    preg_match('/(a)/', $subject, $m->groups);
//                                ^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property groups of a parameter in a pure or external-mutation-free context
    return preg_match('/(b)/', $subject, $out) === 1;
//                                       ^^^^ ImpureByRefAssignment: Assigning to by-reference parameter $out in a @pure function
}
