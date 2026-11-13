===description===
A property passed by reference (`array_shift($o->extra)`) may be emptied, so it loses a
narrowed non-emptiness.
===file===
<?php
final class O {
    /** @var array<string, int> */
    public array $extra = [];
}

function f(O $o): void {
    if ($o->extra) {
        array_shift($o->extra);
        /** @mir-check $o->extra is array<string, int> */
        echo 1;
    }
}
