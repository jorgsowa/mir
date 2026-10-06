===description===
InaccessibleClassConstant does NOT fire when a child class accesses a protected constant from its parent.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    protected const LIMIT = 100;
}

class Child extends Base {
    public function getLimit(): int {
        return Base::LIMIT;
    }
}
