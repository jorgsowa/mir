===description===
`instanceof A || instanceof B` keeps the disjunct that is unrelated to the subject's type
===file===
<?php
interface Ext {}
interface Sub extends Ext {}
interface Other {}

function f(Ext $e): void {
    if ($e instanceof Sub || $e instanceof Other) {
        /** @mir-check $e is Sub|Ext&Other */
        echo 1;
    }
}

function g(Ext $e): void {
    if (!($e instanceof Sub || $e instanceof Other)) {
        return;
    }
    /** @mir-check $e is Sub|Ext&Other */
    echo 1;
}

/** @param (Ext&Other)|Ext $e */
function h(object $e): void {
    if ($e instanceof Sub || $e instanceof Other) {
        /** @mir-check $e is Ext&Other|Sub */
        echo 1;
    }
}
