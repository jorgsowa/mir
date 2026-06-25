===description===
No ReadonlyPropertyAssignment when assigning to another instance of the same class (same declaring scope)
===config===
suppress=MissingConstructor
===file===
<?php
class Config {
    public readonly string $value;

    public function copyFrom(Config $other): void {
        $other->value = $this->value;
    }
}
===expect===
