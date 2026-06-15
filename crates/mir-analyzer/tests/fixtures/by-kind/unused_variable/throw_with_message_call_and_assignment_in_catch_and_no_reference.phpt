===description===
Throw with message call and assignment in catch and no reference
===config===
suppress=MissingThrowsDocblock
===file===
<?php
function dangerous(): string {
    if (rand(0, 1)) {
        throw new Exception("bad");
    }

    return "hello";
}

function callDangerous(): void {
    $s = null;

    try {
        dangerous();
    } catch (Exception $e) {
        echo $e->getMessage();
        $s = "hello";
    }
}
===expect===
UnusedVariable@11:4-11:6: Variable $s is never read
