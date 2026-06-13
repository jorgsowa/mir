===description===
No ImplicitToStringCast when class has __toString method
===config===
suppress=UnusedVariable
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
