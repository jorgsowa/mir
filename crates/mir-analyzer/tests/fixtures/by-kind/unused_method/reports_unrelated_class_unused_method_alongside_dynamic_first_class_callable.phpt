===description===
A dynamic first-class-callable on one class must not blanket-exempt an
unrelated class's private method.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
//<^ +6:1 UnusedClass: Class Foo is never referenced
    private function helper(): void {}

    public function run(string $name): callable {
        return $this->$name(...);
    }
}

class Bar {
//<^ +2:1 UnusedClass: Class Bar is never referenced
    private function unused(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Bar::unused() is never called
}
