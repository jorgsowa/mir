===description===
Throw with return in one catch and no reference
===file===
<?php
class E1 extends Exception {}

function dangerous(): void {
    if (rand(0, 1)) {
        throw new Exception("bad");
    }
}

function callDangerous(): void {
    try {
        dangerous();
        $s = true;
    } catch (E1 $e) {
        echo $e->getMessage();
        $s = false;
    } catch (Exception $e) {
        return;
    }
}
===expect===
UnusedVariable@13:9-13:11: Variable $s is never read
