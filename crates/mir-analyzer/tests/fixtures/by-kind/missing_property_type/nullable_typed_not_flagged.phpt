===description===
MissingPropertyType does NOT fire for properties with nullable type hints — ?T is a valid native type.
===file===
<?php
class User {
    public ?string $nickname = null;
    protected ?int $parentId = null;
}
===expect===
