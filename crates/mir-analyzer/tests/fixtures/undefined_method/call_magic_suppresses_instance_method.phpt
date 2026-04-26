===file===
<?php
class Magic {
    public function __call(string $name, array $arguments): mixed {
        return null;
    }
}
function test(): void {
    $m = new Magic();
    $m->anything();
    $m->anotherMissing(1, 2);
}
===expect===
