===source===
<?php
class Foo {
    public int $value = 0;
}
function test(?Foo $obj): void {
    echo $obj->value;
}
===expect===
PossiblyNullPropertyFetch: $obj->value
