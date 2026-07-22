===description===
When BOTH branches of an if/else write the same readonly property, it IS
definitely initialized on every path reaching the merge point — a further
write after the merge is a certain re-init and must be flagged, unlike the
single-branch case.
===config===
suppress=MissingConstructor
===file===
<?php
class Box {
    public readonly int $value;

    public function init(bool $cond, int $v): void {
        if ($cond) {
            $this->value = $v;
        } else {
            $this->value = $v + 1;
        }
        $this->value = $v + 2;
    }
}
===expect===
ReadonlyPropertyAlreadyInitialized@11:8-11:29: Cannot modify readonly property Box::$value — already initialized
