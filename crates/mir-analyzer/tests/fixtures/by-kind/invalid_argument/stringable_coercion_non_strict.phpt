===description===
Passing a Stringable object where string is expected is allowed in non-strict mode (PHP coerces via __toString())
===file===
<?php
class Tag implements Stringable {
    public function __toString(): string {
        return '<b>bold</b>';
    }
}

function render(string $html): void {}

// Should NOT report InvalidArgument — PHP calls __toString() in coercive mode.
render(new Tag());
===expect===
UnusedParam@8:16-8:28: Parameter $html is never used
