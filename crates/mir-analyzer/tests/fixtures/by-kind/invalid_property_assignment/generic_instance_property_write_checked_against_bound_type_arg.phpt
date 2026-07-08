===description===
Writing to a `@template T`-typed property through a receiver whose type args
are statically known (from a constructor call) must be checked against the
bound concrete type, not waved through just because the docblock type
mentions a template name.
===config===
suppress=MissingPropertyType
===file===
<?php

/**
 * @template T
 */
class Box {
    /** @var T */
    public $value;

    /** @param T $value */
    public function __construct($value) {
        $this->value = $value;
    }
}

function bad(): void {
    $b = new Box(42);
    $b->value = 'not an int';
}
===expect===
InvalidPropertyAssignment@18:4-18:28: Property $value expects 'int', cannot assign '"not an int"'
