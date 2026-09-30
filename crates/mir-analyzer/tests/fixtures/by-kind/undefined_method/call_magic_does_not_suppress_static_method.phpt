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
//  ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Magic::missing() does not exist
}
===expect===
