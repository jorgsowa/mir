===description===
FP-K6: PHP routes a read of an inaccessible (private or protected) property
through a declared `__get()` instead of raising an error — verified live,
`$b->value`/`$b->secret` below both succeed and call `__get`. mir emitted
InaccessibleProperty unconditionally, never checking for the magic fallback.
===config===
suppress=UnusedParam
===file===
<?php
class Box {
    protected string $value = 'x';
    private string $secret = 'y';

    public function __get(string $name): mixed {
        return "magic:$name";
    }
}

function readValue(Box $b): mixed {
    return $b->value;
}

function readSecret(Box $b): mixed {
    return $b->secret;
}
===expect===
