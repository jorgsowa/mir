===description===
@var T $obj->prop refines the property on a parameter and on a local, without unused-variable noise
===file===
<?php
class Box {
    public mixed $prop = null;
}
function p(Box $obj): void {
    /**
     * @var string $obj->prop
     * @mir-check $obj->prop is string
     */
    echo $obj->prop;
}
function q(): void {
    $local = new Box();
    /**
     * @var list<int> $local->prop
     * @mir-check $local->prop is list<int>
     */
    echo count($local->prop);
}
===expect===
