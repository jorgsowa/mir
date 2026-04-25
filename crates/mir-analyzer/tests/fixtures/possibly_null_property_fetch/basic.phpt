===file===
<?php
class Foo {
    public int $value = 0;
}
function test(?Foo $obj): void {
    echo $obj->value;
}
===expect===
PossiblyNullPropertyFetch: Cannot access property $value on possibly null value
