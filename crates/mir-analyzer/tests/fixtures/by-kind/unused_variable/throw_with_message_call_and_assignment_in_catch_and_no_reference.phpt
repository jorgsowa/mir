===description===
Throw with message call and assignment in catch and no reference
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
UnusedVariable
===ignore===
TODO
