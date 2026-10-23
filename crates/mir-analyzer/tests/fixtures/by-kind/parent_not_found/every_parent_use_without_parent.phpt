===description===
Every `parent` use in a class without a parent is reported once: method call,
constant, `::class`, static property and `new parent`. A trait is not checked,
since `parent` there refers to the using class's parent.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Q {
    public function f() {
        parent::g();
//      ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
        echo parent::X;
//           ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
        echo parent::class;
//           ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
        echo parent::$p;
//           ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
        return new parent();
//                 ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
    }
}

trait T {
    public function f() {
        parent::g();
        return [parent::X, parent::class, parent::$p, new parent()];
    }
}
