===description===
A mixed value assigned to a nullable property keeps it mixed, not the nullable declared type
===file===
<?php
final class Box {
    private ?string $h = null;
    private static ?string $s = null;
    private ?string $other = null;
    private int $n = 0;
    private mixed $untyped = null;

    public function instance(mixed $m): string {
        $this->h = $m;
        /** @mir-check $this->h is mixed */
        $this->h;
        return $this->h;
//      ^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
    }

    public function viaStatic(mixed $m): string {
        self::$s = $m;
        return self::$s;
//      ^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
    }

    public function nonNullableDeclared(mixed $m): int {
        $this->n = $m;
        return $this->n;
    }

    public function untypedDeclared(mixed $m): mixed {
        $this->untyped = $m;
        return $this->untyped;
    }

    public function otherPropertyUntouched(mixed $m): string {
        $this->h = $m;
        return $this->other;
//      ^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function nullAssignmentStillNullable(mixed $m): string {
        $this->h = $m;
        $this->h = null;
        return $this->h;
//      ^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'null' is not compatible with declared 'string'
    }

    public function laterMixedAssignmentClearsNull(mixed $m): string {
        $this->h = null;
        $this->h = $m;
        return $this->h;
//      ^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
    }
}
