===description===
A private property reachable only via a dynamic property access
($this->$name) elsewhere on the class must not be reported unused, since
the exact target isn't statically known.
===config===
suppress=
===file===
<?php
class Foo {
    private string $secret = 'x';

    public function get(string $name): mixed {
        return $this->$name;
    }
}
===expect===
