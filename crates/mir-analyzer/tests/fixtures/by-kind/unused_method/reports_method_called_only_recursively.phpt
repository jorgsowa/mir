===description===
A private method whose only call is inside its own body is unused
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class A {
    private function countdown(int $n): int {
//  ^ +2:5 UnusedMethod: Private method A::countdown() is never called
        return $n > 0 ? $this->countdown($n - 1) : 0;
    }

    private function viaClosure(int $n): callable {
//  ^ +2:5 UnusedMethod: Private method A::viaClosure() is never called
        return fn() => $this->viaClosure($n);
    }
}

new A();
