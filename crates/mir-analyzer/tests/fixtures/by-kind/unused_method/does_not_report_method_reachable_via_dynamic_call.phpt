===description===
A private method reachable only via a dynamic method call ($this->$name())
elsewhere on the class must not be reported unused, since the exact target
isn't statically known.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
//<^ +6:1 UnusedClass: Class Foo is never referenced
    private function helper(): void {}

    public function run(string $name): void {
        $this->$name();
    }
}
