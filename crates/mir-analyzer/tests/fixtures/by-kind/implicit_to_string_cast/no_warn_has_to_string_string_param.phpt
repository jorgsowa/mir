===description===
No warning when object with __toString (no \Stringable) is passed to a string parameter
===config===
suppress=UnusedParam
===file===
<?php
class Label {
    public function __toString(): string { return 'Label'; }
}

function render(string $s): void {}

render(new Label());
===expect===
