===description===
A builtin declared once per overload in the stubs accepts the arguments of either signature.
===file===
<?php
/** @param array{expires?: int, path?: string} $options */
function cookies(array $options, \SessionHandlerInterface $handler): bool {
    return setcookie('a', 'b', $options)
        && setcookie('a', 'b', 0, '/')
        && session_set_save_handler($handler, false);
}

function wrongType(): bool {
    return setcookie('a', 'b', 'tomorrow');
//                             ^^^^^^^^^^ InvalidArgument: Argument $expires_or_options of setcookie() expects 'int|array', got '"tomorrow"'
}
