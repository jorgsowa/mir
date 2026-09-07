===description===
Array destructuring (`[key => $v] = $arr`) re-implemented String/Int-only
key resolution instead of reusing `literal_array_key_of_kind` — a
bool/float/null destructure key fell back to `mixed` instead of resolving
the canonical slot.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function destructureBoolKeyResolvesToCanonicalSlot(): void {
    $arr = [1 => 'x', 0 => 'y'];
    [true => $v] = $arr;
    /** @mir-check $v is 'x' */
    $_ = $v;
}

function destructureNullKeyResolvesToCanonicalSlot(): void {
    $arr = ['' => 'empty', 'x' => 'other'];
    [null => $v] = $arr;
    /** @mir-check $v is 'empty' */
    $_ = $v;
}

function destructureFloatKeyResolvesToCanonicalSlot(): void {
    $arr = [1 => 'x'];
    [1.9 => $v] = $arr;
    /** @mir-check $v is 'x' */
    $_ = $v;
}
===expect===
