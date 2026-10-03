===description===
Deprecated clone method with call
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    /**
     * @deprecated
     */
    public function __clone() {
    }
}

$a = new Foo;
$aa = clone $a;
//    ^^^^^^^^ DeprecatedMethodCall: Call to deprecated method Foo::__clone
===expect===
