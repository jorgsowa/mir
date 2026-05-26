===description===
Throw with message call and nested assignment in try and catch and no reference
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

    if (rand(0, 1)) {
        $s = "hello";
    } else {
        try {
            $t = dangerous();
        } catch (Exception $e) {
            echo $e->getMessage();
            $t = "hello";
        }

        if ($t) {
            $s = $t;
        }
    }
}
===expect===
UnusedVariable
===ignore===
TODO
