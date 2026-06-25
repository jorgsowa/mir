===description===
RawObjectIteration does NOT fire for Generator (implements Iterator extends Traversable)
===file===
<?php
function gen(): \Generator {
    yield 1;
    yield 2;
}

function process(): void {
    yield from gen();
}
===expect===
