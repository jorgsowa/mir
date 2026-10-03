===description===
A @var naming a variable that is never assigned documents an externally
provided variable; it is not a dead write.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo { public function m(): int { return 1; } }

function provided(): int {
    /** @var Foo $injected */
    return $injected->m();
}

function unused_annotation(): void {
    /** @var Foo $ghost */
    echo 1;
}

function property_annotation(Foo $o): void {
    /** @var int $o->count */
    echo $o->m();
}
===expect===
