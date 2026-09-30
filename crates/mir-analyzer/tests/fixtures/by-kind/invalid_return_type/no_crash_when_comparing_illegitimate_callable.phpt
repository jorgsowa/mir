===description===
No crash when comparing illegitimate callable
===file===
<?php
class C {}

function foo() : C {
    return fn (int $i) => "";
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'Closure(int): ""' is not compatible with declared 'C'
}
===expect===
