===description===
A `@param` docblock type for a non-scalar (object/array/class) param must
not drop the native hint's own nullability — PHP enforces `?object`
regardless of what the docblock says, unlike the scalar family, which
already had an explicit conflict guard. Covers both null-check directions
(`===`/`!==`) on the same variable.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param object ...$objects */
function validate(?object ...$objects): void {
//                           ^^^^^^^^ MismatchingDocblockParamType: Docblock type 'object' for $objects does not match inferred 'object|null'
    foreach ($objects as $obj) {
        if ($obj === null || $obj->foo) {
            $id = $obj !== null ? $obj->id : 'unknown';
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $id is assigned a mixed type
        }
    }
}
