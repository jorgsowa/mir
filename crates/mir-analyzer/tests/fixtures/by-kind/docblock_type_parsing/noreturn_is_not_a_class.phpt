===description===
`noreturn` in any letter case is a keyword, not a class in the current namespace.
===file===
<?php
namespace App;

/** @return noreturn */
function stop() {
    throw new \LogicException('stop');
}

/** @return NoReturn */
function quit() {
    exit(2);
}
