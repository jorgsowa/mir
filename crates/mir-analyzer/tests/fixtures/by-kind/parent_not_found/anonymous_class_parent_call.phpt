===description===
FP: parent:: inside an anonymous class method that extends a named class must
not emit ParentNotFound. Anonymous classes are not in the collector DB, so the
parent FQCN must be resolved from the AST extends clause.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

class Base {
    public function hello(): string { return 'hello'; }
    protected function compute(): int { return 42; }
}

$obj = new class extends Base {
    public function hello(): string {
        return parent::hello() . ' world';
    }

    public function getComputed(): int {
        return parent::compute();
    }
};
===expect===
