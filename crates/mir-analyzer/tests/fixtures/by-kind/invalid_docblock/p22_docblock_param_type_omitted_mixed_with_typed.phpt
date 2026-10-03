===description===
P22 (mixed docblock): a type-omitted `@param $name` line and a normally-typed `@param
Type $name` line in the same docblock must each resolve to their own name/type — the
fix to the omitted-type case must not disturb the existing typed-param path.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

/**
 * @param $ids the ids to look up, no type given
 * @param string $label a normally-typed param
 */
function f($ids, $label): void {
    /** @mir-check $ids is mixed */
    /** @mir-check $label is string */
    echo $label;
}
===expect===
