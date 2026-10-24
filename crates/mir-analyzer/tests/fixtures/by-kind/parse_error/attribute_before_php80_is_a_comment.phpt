===description===
Before PHP 8.0 #[...] starts a line comment, so attributes are not errors.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
#[\Attribute]
class A {
    #[\ReturnTypeWillChange]
    public function f(): int { return 1; }
}
$x = 1; #[inline comment]
echo $x;
