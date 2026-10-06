===description===
A mixed value assigned to a nullable property refines it to the non-null declared type
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
        /** @mir-check $this->h is string */
        $this->h;
        return $this->h;
    }

    public function viaStatic(mixed $m): string {
        self::$s = $m;
        return self::$s;
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
    }
}
===expect===
