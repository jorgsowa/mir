===description===
A call may run a closure that captured an object, so the object's narrowed properties are forgotten.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function viaCallback(string $s): void {
    $state = new stdClass();
    $state->vars = [];
    $state->getVars = static function (array $m) use ($state): string {
        $state->vars[] = $m[0];
        return '';
    };
    $state->vars = [];
    preg_replace_callback('#x#', $state->getVars, $s);
    if ($state->vars) {
        echo $state->vars[0];
    }
}
class Box { public ?int $n = null; }
function viaArrowFunction(Box $o): void {
    $f = static fn (): int => $o->n = 1;
    $o->n = null;
    $f();
    /** @mir-check $o->n is int|null */
    echo 1;
}
