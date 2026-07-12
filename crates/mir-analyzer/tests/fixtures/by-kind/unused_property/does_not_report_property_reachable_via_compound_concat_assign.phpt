===description===
A private property whose only reference is a compound concat assign
($this->log .= 'x') must not be reported unused.
===config===
suppress=
===file===
<?php
class Foo {
    private string $log = '';

    public function run(): void {
        $this->log .= 'x';
    }
}
===expect===
