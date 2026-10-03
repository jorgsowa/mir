===description===
A method declared `: self` returns the declaring class, so a subclass-only call on the result is reported against the declaring class.
===config===
<mir>
  <issueHandlers>
    <UnusedMethod errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    public function returnsSelf(): self { return $this; }
    public static function make(): self { return new self(); }
}

class Sub extends Base {
    public function subOnly(): void {}
}

(new Sub())->returnsSelf()->nope();
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Base::nope() does not exist
(new Sub())->returnsSelf()->subOnly();
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Base::subOnly() does not exist
Sub::make()->subOnly();
//<^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Base::subOnly() does not exist
===expect===
