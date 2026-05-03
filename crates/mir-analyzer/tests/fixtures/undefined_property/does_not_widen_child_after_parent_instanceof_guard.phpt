===description===
does not widen child after parent instanceof guard
===file===
<?php
class Base {}
class Child extends Base {
    public string $childOnly;
}
class Other {}
/** @param Child|Other $value */
function test(object $value): void {
    if (!$value instanceof Base) {
        return;
    }
    echo $value->childOnly;
}
===expect===
===ignore===
TODO
