===description===
Reading `$arr[key]` hand-rolled String/Int-only literal key resolution
instead of reusing `literal_array_key_of_kind` — a bool/float/null index
fell back to merging every shape property (wrong value type) and could
silently skip a `NonExistentArrayOffset` diagnostic that should fire.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function readBoolKeyResolvesToCanonicalSlot(): void {
    $arr = [1 => 'x', 0 => 'y'];
    $v = $arr[true];
    /** @mir-check $v is 'x' */
    $_ = $v;
}

function readMissingBoolKeyIsFlagged(): void {
    $arr = ['a' => 1];
    $v = $arr[true];
//            ^^^^ NonExistentArrayOffset: Array offset '1' does not exist
}

function readNullKeyResolvesToCanonicalSlot(): void {
    $arr = ['' => 'empty', 'x' => 'other'];
    $v = $arr[null];
    /** @mir-check $v is 'empty' */
    $_ = $v;
}

function readFloatKeyResolvesToCanonicalSlot(): void {
    $arr = [1 => 'x', 2 => 'y'];
    $v = $arr[1.9];
//            ^^^ ImplicitFloatToIntCast: Implicit cast from 1.9 to int truncates the fractional part
    /** @mir-check $v is 'x' */
    $_ = $v;
}
===expect===
