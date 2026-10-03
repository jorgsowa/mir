===description===
A `__call` on any intersection member answers an otherwise-unknown method
for the whole intersection, whichever position the member is in.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface A { public function a(): int; }
class Magic { public function __call($n, $args) { return 1; } }

function magicSecond(A&Magic $x): void {
    $r = $x->zzz();
    /** @mir-check $r is mixed */
    $_ = $r;
}

function magicFirst(Magic&A $x): void {
    $r = $x->zzz();
    /** @mir-check $r is mixed */
    $_ = $r;
}

function declaredWinsOverMagic(A&Magic $x): void {
    $r = $x->a();
    /** @mir-check $r is int */
    $_ = $r;
}

/** @param (A&Magic)|null $x */
function nullsafe($x): void {
    $x?->zzz();
}
===expect===
