===description===
`object{prop: Type, ...}` (Psalm's object-shape syntax) is approximated as
plain `object` instead of being misparsed as a bogus named class.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param object{name: string, age: int} $x */
function takeShape($x): void {
    /** @mir-check $x is object */
    $_ = 1;
}

/** @return object{ok: bool} */
function returnsShape() {
    return (object) ['ok' => true];
}
