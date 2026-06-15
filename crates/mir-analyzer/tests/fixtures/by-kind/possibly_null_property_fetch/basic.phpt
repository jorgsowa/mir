===description===
Basic
===file===
<?php
class Foo {
    public int $value = 0;
}
function test(?Foo $obj): void {
    echo $obj->value;
}
===expect===
PossiblyNullPropertyFetch@6:9-6:20: Cannot access property $value on possibly null value
