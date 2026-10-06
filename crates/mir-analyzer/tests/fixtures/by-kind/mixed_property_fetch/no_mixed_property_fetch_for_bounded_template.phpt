===description===
G1: property fetch on a bounded template param (T of object) must not emit MixedPropertyFetch —
bounds restrict T to objects, so the fetch is valid.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T of object
 * @param T $obj
 */
function fetch_prop($obj): void {
    // T of object — not mixed, must not fire MixedPropertyFetch
    $obj->name;
    $obj->value;
}
