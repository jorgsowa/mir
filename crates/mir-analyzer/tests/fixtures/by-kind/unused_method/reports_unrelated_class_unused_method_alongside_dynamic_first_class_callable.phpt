===description===
A dynamic first-class-callable on one class must not blanket-exempt an
unrelated class's private method.
===config===
suppress=
===file===
<?php
class Foo {
    private function helper(): void {}

    public function run(string $name): callable {
        return $this->$name(...);
    }
}

class Bar {
    private function unused(): void {}
}
===expect===
UnusedMethod@11:4-11:38: Private method Bar::unused() is never called
