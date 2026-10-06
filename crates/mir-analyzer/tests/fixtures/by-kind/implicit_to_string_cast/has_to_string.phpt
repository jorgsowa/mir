===description===
No ImplicitToStringCast when class has __toString method
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
