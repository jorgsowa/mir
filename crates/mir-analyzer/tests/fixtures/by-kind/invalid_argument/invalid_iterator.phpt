===description===
Invalid iterator
===ignore===
TODO
===file===
<?php
function example() : int {
    return 0;
}

function example2() : Generator {
    yield from example();
}
===expect===
