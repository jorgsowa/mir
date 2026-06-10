===description===
bare return; inside a generator is valid — it terminates the iteration
===file===
<?php
function earlyExit(): \Generator {
    if (rand(0, 1)) {
        return;
    }
    yield 1;
}

function earlyExitBeforeYield(): \Generator {
    if (rand(0, 1)) {
        return;
    }
    yield from [];
}

function yieldFirst(): \Generator {
    yield 1;
    return;
}
===expect===
