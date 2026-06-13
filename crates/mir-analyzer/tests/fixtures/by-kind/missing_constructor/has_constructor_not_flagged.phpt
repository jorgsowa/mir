===description===
MissingConstructor does NOT fire when the class defines a constructor, even if
properties are non-nullable.
===file===
<?php
class WithConstructor {
    public string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}

new WithConstructor("hello");

===expect===
