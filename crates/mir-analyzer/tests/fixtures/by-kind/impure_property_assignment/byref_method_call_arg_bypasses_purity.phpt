===description===
Same by-ref-argument gap again, but through a method call -- method calls'
by-ref write-back only ever ran for a param declaring `@param-out`, and
even then only matched `ExprKind::Variable`, so a property argument was
never checked regardless.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    public $n = 0;
}

class Helper {
    public function bump(int &$n): void {
        $n++;
    }
}

/** @pure */
function run(Box $b, Helper $h): void {
    $h->bump($b->n);
//  ^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method bump() in a pure or immutable context
//           ^^^^^ ImpurePropertyAssignment: Assigning to property n of a parameter in a pure or external-mutation-free context
}
