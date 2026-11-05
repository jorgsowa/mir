===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
interface Box {
    /** @return T */
    public function get(): mixed;
}

function user_class(Box $b): mixed {
    $x = $b->get();
    /** @mir-check $x is mixed */
    return $x;
}

function stub_class(\Iterator $i): string {
    $x = $i->current();
    if (!is_string($x)) {
        if (!is_scalar($x)) {
            return '';
        }
        $x = (string) $x;
    }
    return $x;
}
