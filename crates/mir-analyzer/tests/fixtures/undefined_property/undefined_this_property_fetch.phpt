===description===
undefinedThisPropertyFetch
===file===
<?php
class A {
    public function fooFoo(): void {
        echo $this->foo;
    }
}
===expect===
UndefinedThisPropertyFetch
===ignore===
TODO
