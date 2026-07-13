===description===
Sibling of init_in_child_constructor: a subclass constructor may still init its own readonly property.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class Base {
    public readonly string $name;
}

class Child extends Base {
    public readonly string $label;

    public function __construct(string $name, string $label) {
        $this->label = $label;
    }
}
===expect===
