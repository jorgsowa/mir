===description===
FN: assigning to an existing shape key widened the entire shape into a
generic array, discarding the precise types of all its OTHER keys.
===config===
suppress=UnusedVariable
===file===
<?php
function f(): void {
    $arr = ['a' => 1, 'b' => 'str'];
    $arr['a'] = 2;
    $b = $arr['b'];
    /** @mir-check $b is string */
    $_ = $b;
}
===expect===
