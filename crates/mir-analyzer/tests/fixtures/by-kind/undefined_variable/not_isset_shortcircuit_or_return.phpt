===description===
!isset short-circuit with || operator — guard clause pattern
Common PHP idiom: !isset($x) || call($x) should not error on UndefinedVariable in RHS
===file===
<?php
function doSomething($x): void { echo $x; }
function test() {
    !isset($x) || doSomething($x);
    // After fix: $x should be narrowed as defined in RHS
}
===expect===
