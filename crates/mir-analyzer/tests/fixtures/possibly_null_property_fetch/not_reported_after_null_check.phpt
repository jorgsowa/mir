===description===
not reported after null check
===file===
<?php
class Foo {
    public int $value = 0;
}
function test(?Foo $obj): void {
    if ($obj !== null) {
        /** @mir-check $obj is Foo */
        echo $obj->value;
    }
}
===expect===
