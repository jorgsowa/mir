===description===
No ImplicitToStringCast when class has __toString method
===file===
<?php
class Foo {
    public function __toString() {
        return 'foo';
    }
}
$f = new Foo();
$s = 'Value: ' . $f;
echo $f;
===expect===
