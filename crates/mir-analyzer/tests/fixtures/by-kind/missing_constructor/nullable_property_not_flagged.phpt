===description===
MissingConstructor does NOT fire when all uninitialized properties are nullable
— they implicitly default to null.
===file===
<?php
class AllNullable {
    public ?string $name;
    public ?int $age;
}

new AllNullable();

===expect===
