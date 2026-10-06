===description===
Template bounds reject backslash-qualified keywords.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T of \int
//                ^^^^ InvalidDocblockType: Invalid docblock type: @template backslash-qualified non-class type '\int' is not a fully qualified name
 * @param T $a
 */
function f($a): void {
}
