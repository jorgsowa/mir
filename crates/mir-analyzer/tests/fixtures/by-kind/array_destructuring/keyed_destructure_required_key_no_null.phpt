===description===
Sibling of keyed_destructure_optional_key_includes_null: a required
(non-optional) shape key stays exactly its declared type, no null.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array{a: string} $arr
 */
function test(array $arr): void {
    ['a' => $a] = $arr;
    /** @trace $a */
    strlen($a);
//  ^^^^^^^^^^^ Trace: Type of $a is string
}
===expect===
