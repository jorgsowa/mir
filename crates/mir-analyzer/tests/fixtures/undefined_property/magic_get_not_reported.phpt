===source===
<?php
class Magic {
    /** @param string $name */
    public function __get($name): mixed { return null; }
}
function test(): void {
    $m = new Magic();
    echo $m->anything;
}
===expect===
# __get suppresses UndefinedProperty — no UndefinedProperty emitted.
# UnusedParam on __get's $name is a known false positive (magic methods
# receive the param from the PHP runtime, not from user call sites).
UnusedParam: $name
