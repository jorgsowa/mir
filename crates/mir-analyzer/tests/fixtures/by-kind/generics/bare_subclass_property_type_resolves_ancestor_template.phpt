===description===
A bare subclass that doesn't redeclare @template (`class IntBox extends Box
{}`) still resolves an inherited `@var T` property through the ancestor's
template the same way a directly-generic class already does.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    public $value;
}

class IntBox extends Box {}

function test(): void {
    /** @var IntBox<int> $box */
    $box = new IntBox();
    /** @mir-check $box->value is int */
    $_ = $box->value;
}
===expect===
