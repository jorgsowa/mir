===description===
A value the property accepts once `null` or `false` is removed, or in part, is reported under a `Possibly*` kind.
===file===
<?php
class C {
    public int $n = 0;
    public static int $s = 0;
    public function f(?int $nullable, int|false $falsable, int|string $either): void {
        $this->n = $nullable;
//      ^^^^^^^^^^^^^^^^^^^^ PossiblyNullPropertyAssignmentValue: Property $n expects 'int', the assigned 'int|null' might be null
        $this->n = $falsable;
//      ^^^^^^^^^^^^^^^^^^^^ PossiblyFalsePropertyAssignmentValue: Property $n expects 'int', the assigned 'int|false' might be false
        $this->n = $either;
//      ^^^^^^^^^^^^^^^^^^ PossiblyInvalidPropertyAssignmentValue: Property $n expects 'int', the assigned 'int|string' might not match
        self::$s = $nullable;
//      ^^^^^^^^^^^^^^^^^^^^ PossiblyNullPropertyAssignmentValue: Property $s expects 'int', the assigned 'int|null' might be null
        $this->n = 'x';
//      ^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $n expects 'int', cannot assign '"x"'
    }
}
