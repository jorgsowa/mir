===description===
Negative control for the K6 fix: without a declared `__get()`, an
inaccessible protected property read must still be flagged — the magic-get
fallback check must not suppress the diagnostic for classes that don't
actually define it.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    protected string $value = 'x';
}

function readValue(Box $b): string {
    return $b->value;
//             ^^^^^ InaccessibleProperty: Cannot access property Box::$value
}
