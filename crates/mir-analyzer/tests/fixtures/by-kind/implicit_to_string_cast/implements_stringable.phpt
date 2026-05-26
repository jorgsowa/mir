===description===
No ImplicitToStringCast when class implements Stringable interface
===file===
<?php
// Stringable is a built-in PHP interface since PHP 8.0
class Foo implements Stringable {
    public function __toString(): string {
        return 'foo';
    }
}
$f = new Foo();
$s = 'Value: ' . $f;
echo $f;
===expect===
