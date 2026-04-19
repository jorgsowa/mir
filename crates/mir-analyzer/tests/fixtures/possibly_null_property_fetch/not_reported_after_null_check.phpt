===source===
<?php
class Foo {
    public int $value = 0;
}
function test(?Foo $obj): void {
    if ($obj !== null) {
        echo $obj->value;
    }
}
===expect===
