===description===
Prevent void to null conversion signature
===file===
<?php
class A {
    public function foo(): ?string {
        return rand(0, 1) ? "hello" : null;
    }
}

class B extends A {
    public function foo(): void {
        return;
    }
}
===expect===
MethodSignatureMismatch@9:4-9:33: Method B::foo() signature mismatch: return type 'void' is not a subtype of parent 'string|null'
