===description===
A compound concat assign to one property does not exempt an unrelated unused
property on the same class.
===config===
suppress=
===file===
<?php
class Foo {
    private string $log = '';
    private string $unused = '';

    public function run(): void {
        $this->log .= 'x';
    }
}
===expect===
UnusedProperty@4:4-4:31: Private property Foo::$unused is never read
