===description===
No ImplicitToStringCast when parent class has __toString
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class ParentClass {
    public function __toString() {
        return 'parent';
    }
}

class Child extends ParentClass {}

$c = new Child();
$s = 'Value: ' . $c;
echo $c;
