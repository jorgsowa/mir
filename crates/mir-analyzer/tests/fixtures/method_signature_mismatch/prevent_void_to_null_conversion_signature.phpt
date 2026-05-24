===description===
preventVoidToNullConversionSignature
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
MethodSignatureMismatch
===ignore===
TODO
