===description===
No crash when comparing illegitimate callable
===file===
<?php
class C {}

function foo() : C {
    return fn (int $i) => "";
}
===expect===
InvalidReturnType@5:5-5:30: Return type 'Closure(int): ""' is not compatible with declared 'C'
