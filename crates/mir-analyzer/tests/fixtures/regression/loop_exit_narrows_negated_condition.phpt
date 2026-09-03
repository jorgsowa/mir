===description===
A non-break exit from a while/for/do-while loop can only happen once the
loop's own condition evaluates false — analyze_loop_widened never applied
that negated narrowing to the post-loop state, so a variable proven
non-null only by the loop's own guard stayed widened to its pre-loop type
after the loop, even though every reachable exit path proves it null.
===file===
<?php

function whileExit(?string $x): void {
    while ($x !== null) {
        $x = null;
    }
    /** @mir-check $x is null */
    echo "";
}

function forExit(?string $x): void {
    for (; $x !== null;) {
        $x = null;
    }
    /** @mir-check $x is null */
    echo "";
}

function doWhileExit(?string $x): void {
    do {
        echo "iter";
    } while ($x !== null);
    /** @mir-check $x is null */
    echo "";
}
===expect===
