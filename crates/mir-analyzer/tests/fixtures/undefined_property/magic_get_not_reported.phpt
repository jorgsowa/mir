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
UnusedParam: $name
