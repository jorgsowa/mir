===description===
A private property must still be reported unused even when a private method
of the same name is called elsewhere on the class (regression: the two used
to share one reference-index key and the method's usage hid the property).
===file===
<?php
class Foo {
    private string $bar = 'x';

    private function bar(): string {
        return 'y';
    }

    public function run(): string {
        return $this->bar();
    }
}
===expect===
UnusedProperty@3:4-3:29: Private property Foo::$bar is never read
