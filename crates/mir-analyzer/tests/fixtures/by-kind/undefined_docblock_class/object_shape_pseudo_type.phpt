===description===
`object{prop: Type, ...}` (Psalm's object-shape syntax) is an object shape,
not a named class.
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
    /** @mir-check $x is object{name: string, age: int} */
    $_ = 1;
}

/** @return object{ok: bool} */
function returnsShape() {
    return (object) ['ok' => true];
}
