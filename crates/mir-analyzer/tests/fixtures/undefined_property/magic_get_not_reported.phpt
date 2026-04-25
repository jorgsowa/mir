===file===
<?php
class Magic {
    public function __get(string $name): mixed {
        return null;
    }
}
function test(): void {
    $m = new Magic();
    echo $m->anything;
}
===expect===
