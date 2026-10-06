===description===
The shorthand @mutation-free tag (without the psalm- prefix) is recognized.
===file===
<?php

class Token {
    public string $value = '';

    /** @mutation-free */
    public function setValue(string $v): void {
        $this->value = $v;
//      ^^^^^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property value of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
    }
}
