===file===
<?php
class Magic {
    public function __call(string $name, array $arguments): mixed {
        return null;
    }
}
function test(): void {
    Magic::missing();
}
===expect===
UndefinedMethod: Method Magic::missing() does not exist
