===description===
An open array shape that passed `is_callable()` is not rejected for lacking exactly two entries
===file===
<?php
function check(callable $c): callable { return $c; }

function f(array $controller): callable {
    $controller[0] = new stdClass();
    if (!is_callable($controller)) {
        throw new InvalidArgumentException('not callable');
    }
    return check($controller);
}
