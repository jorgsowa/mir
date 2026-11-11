===description===
A non-falsy-string is always truthy; a non-empty-string may be "0" and is not.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param non-falsy-string $name
 * @param non-empty-string $label
 */
function run(string $name, string $label): void {
    if ($name) {
//      ^^^^^ RedundantCondition: Condition is always true, so the check is redundant
        echo $name;
    }
    if ($label) {
        echo $label;
    }
}
