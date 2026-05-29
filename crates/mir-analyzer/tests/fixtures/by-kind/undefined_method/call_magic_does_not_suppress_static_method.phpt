===description===
call magic does not suppress static method
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
UndefinedMethod@8:5-8:21: Method Magic::missing() does not exist
