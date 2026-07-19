===description===
`if ($this->prop = expr)` narrows the property by the assigned value's
truthiness, the property-access counterpart of the plain-variable
assignment-as-condition case.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}

final class Holder {
    public ?Foo $prop = null;

    public function narrowsTrue(): void {
        if ($this->prop = fetch()) {
            /** @mir-check $this->prop is Foo */
            $_ = 1;
        }
    }
}

function fetch(): ?Foo {
    return null;
}
===expect===
