===description===
Cross-file return type mismatch produces InvalidArgument
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Maker.php===
<?php
class Apple {}
class Banana {}
class Maker {
    public function make(): Apple { return new Apple(); }
}
===file:Consumer.php===
<?php
function expect_banana(Banana $v): void {}
expect_banana((new Maker)->make());
//            ^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $v of expect_banana() expects 'Banana', got 'Apple'
