===description===
returning a primitive when a concrete generic class instantiation is declared should error
===file===
<?php
/** @template T */
class Box {}
/** @return Box<string> */
function makeBox(): mixed {
    return 42;
}
===expect===
InvalidReturnType@6:5-6:15: Return type '42' is not compatible with declared 'Box<string>'
