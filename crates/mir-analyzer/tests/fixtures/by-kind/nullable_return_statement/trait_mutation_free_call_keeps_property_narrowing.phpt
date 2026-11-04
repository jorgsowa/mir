===description===
Calling a trait method that writes no property keeps the receiver's property narrowing
===file===
<?php
trait Named {
    private ?string $prefix = null;

    private function name(): string {
        return 'x';
    }

    private function reset(): void {
        $this->prefix = null;
    }
}

final class C {
    use Named;

    public function pure(): string {
        if (null === $this->prefix) {
            return '';
        }
        $this->name();
        return $this->prefix;
    }

    public function impure(): string {
        if (null === $this->prefix) {
            return '';
        }
        $this->reset();
        return $this->prefix;
//      ^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }
}
