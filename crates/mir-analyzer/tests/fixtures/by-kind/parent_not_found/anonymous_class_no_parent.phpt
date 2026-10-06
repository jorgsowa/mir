===description===
ParentNotFound fires when parent:: is used inside an anonymous class that has
no extends clause.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

$obj = new class {
    public function build(): void {
        parent::build();
//      ^^^^^^ ParentNotFound: Cannot use parent:: when current class has no parent
    }
};
